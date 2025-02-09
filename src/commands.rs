use std::collections::HashSet;
use std::{fs, process, time};

use chrono::{offset::Utc, DateTime};
use color_eyre::eyre::{Result, WrapErr};
use color_eyre::owo_colors::OwoColorize;
use colored::{ColoredString, Colorize};
use cookie_store::CookieStore;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use ureq::{Agent, AgentBuilder, Cookie};

use crate::cli::{CachedArgs, CheckArgs};
use crate::parsing::{self, parse_thread, Status, ThreadSlug, HOST};
use crate::utils;

pub fn check(args: &CheckArgs) -> Result<()> {
    if !utils::day_passed_since_last_check() && !args.force {
        utils::warning("One check allowed per day. Use '-f' flag to force another one.");
        process::exit(0);
    }

    let file = &args.file;
    if !file.exists() {
        utils::error("File not found!");
        process::exit(0);
    }

    let text = fs::read_to_string(file).wrap_err("Failed to read file")?;
    let ids = extract_thread_ids(&text);
    let amount = ids.len();

    if amount == 0 {
        utils::error("Detected 0 threads!");
        process::exit(0);
    }

    println!();
    utils::success(&format!("Detected {amount} thread(s)\n"));

    let mut auth_attempt = false;
    let host = url::Url::parse(HOST)?;
    let mut cookiestore = CookieStore::new(None);
    if let Some(token) = &args.xf_user {
        cookiestore.insert_raw(&Cookie::new("xf_user", token), &host)?;
        auth_attempt = true;
    };
    if let Some(token) = &args.xf_tfa_trust {
        cookiestore.insert_raw(&Cookie::new("xf_tfa_trust", token), &host)?;
        auth_attempt = true;
    };

    let agent: Agent = AgentBuilder::new()
        .user_agent(&format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .timeout_read(time::Duration::from_secs(5))
        .timeout_write(time::Duration::from_secs(5))
        .cookie_store(cookiestore)
        .build();

    if auth_attempt && !parsing::logged_in(&agent) {
        utils::error("Authentication failed!");
        std::process::exit(0);
    }

    let bar = ProgressBar::new(amount as u64);
    let template = "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)";
    bar.set_style(ProgressStyle::with_template(template)?.progress_chars("##-"));

    let mut results: Vec<ThreadSlug> = Vec::with_capacity(amount);

    for id in ids {
        results.push(parse_thread(&id, &agent));
        bar.inc(1);
    }
    bar.finish();
    println!();

    let timestamp = Utc::now().timestamp();
    print_check_results(&results);
    cache_results(&results, timestamp)?;
    utils::save_check_timestamp(timestamp);

    Ok(())
}

fn extract_thread_ids(text: &str) -> HashSet<String> {
    let re = Regex::new(r"(?i)/threads/(?:([^./]*)\.)?(\d+)").expect("Regex init error");
    let mut threads: HashSet<String> = HashSet::new();
    for cap in re.captures_iter(text) {
        if let Some(id) = cap.get(2) {
            threads.insert(id.as_str().to_string());
        }
    }
    threads
}

fn print_check_results(results: &[ThreadSlug]) {
    println!(
        "\nGames still in development: {}/{}",
        results
            .iter()
            .filter(|r| r.status == Status::InDevelopment && r.error.is_none())
            .count(),
        results.len()
    );

    print_results_by_status(results, "Completed".bright_blue(), Status::Completed);
    print_results_by_status(results, "Abandoned".yellow(), Status::Abandoned);
    print_results_by_status(results, "On hold".bright_cyan(), Status::OnHold);

    print_error_results(results);
}

#[allow(clippy::needless_pass_by_value)]
fn print_results_by_status(results: &[ThreadSlug], header: ColoredString, status: Status) {
    let iter = results.iter().filter(|r| r.status == status);
    if iter.clone().count() == 0 {
        return;
    }
    println!("\n{header}:");
    for res in iter {
        println!(
            "  - {}\n    Link: {HOST}/threads/{}",
            res.title.bold(),
            res.id
        );
    }
}

fn print_error_results(results: &[ThreadSlug]) {
    let iter = results.iter().filter(|r| r.error.is_some());
    if iter.clone().count() == 0 {
        return;
    }
    println!("\n{}:", "Error".bright_red());
    for res in iter {
        println!(
            "  - {}\n    Error: {}",
            format!("{HOST}/threads/{}", res.id).bold(),
            res.error.clone().unwrap_or_default(),
        );
    }
}

fn cache_results(results: &Vec<ThreadSlug>, timestamp: i64) -> Result<()> {
    let cache_dir = utils::cache_dir();
    let datetime: DateTime<Utc> =
        DateTime::from_timestamp(timestamp, 0).expect("invalid timestamp");
    let fmt_string = datetime.format("%Y-%m-%d %H-%M-%S").to_string();
    let cache_dir = cache_dir.join(fmt_string);
    fs::create_dir_all(&cache_dir).wrap_err("failed to create cache dir")?;
    let file =
        fs::File::create(cache_dir.join("results.json")).wrap_err("failed to create cache file")?;
    serde_json::to_writer(file, results).wrap_err("failed to serialize cache file")?;
    Ok(())
}

pub fn cached(args: &CachedArgs) -> Result<()> {
    let mut offset = 0;
    if let Some(o) = args.offset {
        if o < 0 {
            utils::error("Offset should be positive.");
            process::exit(0);
        }
        offset = o;
    }

    let cache_dir = utils::cache_dir();
    let entries = fs::read_dir(cache_dir)?;
    let mut dirs_with_metadata: Vec<(time::SystemTime, fs::DirEntry)> = Vec::new();
    for entry in entries {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            dirs_with_metadata.push((metadata.created()?, entry));
        }
    }

    if dirs_with_metadata.is_empty() {
        utils::error("No cached results found.");
        process::exit(0);
    }

    dirs_with_metadata.sort_by_key(|&(creation_time, _)| creation_time);
    dirs_with_metadata.reverse();

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let Some((_, entry)) = dirs_with_metadata.get(offset as usize) else {
        utils::error(&format!("Entry with offset {offset} does not exist."));
        process::exit(0);
    };

    let file = entry.path().join("results.json");
    if !file.exists() {
        utils::error("Results file is missing.");
        process::exit(0);
    }

    let file = fs::File::open(file).wrap_err("failed to open cached results file")?;
    let results: Vec<ThreadSlug> =
        serde_json::from_reader(file).wrap_err("failed to deserialize cached results")?;

    print_check_results(&results);

    Ok(())
}

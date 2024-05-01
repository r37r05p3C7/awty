use std::{fs, process, time};
use std::collections::HashSet;

use chrono::{DateTime, offset::Utc};
use color_eyre::eyre::{Result, WrapErr};
use color_eyre::owo_colors::OwoColorize;
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use ureq::{Agent, AgentBuilder};

use crate::cli::{CachedArgs, CheckArgs};
use crate::parsing::{DOMAIN, parse_thread, ParsingResult, Status};
use crate::utils;

pub fn check(args: &CheckArgs) -> Result<()> {
    if !utils::day_passed_since_last_check() && !args.force {
        println!(
            "{}: One check allowed per day. Use '-f' flag to force another one.",
            "Warning".yellow()
        );
        process::exit(0);
    }

    let file = &args.file;
    if !file.exists() {
        println!("{}: File not found!", "Error".red());
        process::exit(0);
    }

    let text = fs::read_to_string(file).wrap_err("Failed to read file")?;
    let ids = extract_thread_ids(&text);
    let amount = ids.len();

    if amount == 0 {
        println!("{}: Detected 0 threads!", "Error".red());
        process::exit(0);
    }

    println!("{}: Detected {} thread(s)", "Success".green(), amount);
    let mut results: Vec<ParsingResult> = Vec::with_capacity(amount);
    let agent: Agent = AgentBuilder::new()
        .user_agent(&format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .timeout_read(time::Duration::from_secs(5))
        .timeout_write(time::Duration::from_secs(5))
        .build();
    let bar = ProgressBar::new(amount as u64);
    let template = "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)";
    bar.set_style(ProgressStyle::with_template(template)?.progress_chars("##-"));

    for id in ids {
        results.push(parse_thread(&id, &agent));
        bar.inc(1);
    }
    bar.finish_and_clear();

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
    return threads;
}

fn print_check_results(results: &Vec<ParsingResult>) {
    println!(
        "\nGames still in development: {}/{}",
        results
            .iter()
            .filter(|r| r.status == Status::InDevelopment && r.error.is_none())
            .count(),
        results.len()
    );

    print_results_by_status(&results, "Completed".bright_blue(), Status::Completed);
    print_results_by_status(&results, "Abandoned".yellow(), Status::Abandoned);
    print_results_by_status(&results, "On hold".bright_cyan(), Status::OnHold);

    print_error_results(&results);
}

fn print_results_by_status(results: &Vec<ParsingResult>, header: ColoredString, status: Status) {
    let iter = results.iter().filter(|r| r.status == status);
    if iter.clone().count() == 0 {
        return;
    }
    println!("\n{}:", header);
    for res in iter {
        println!(
            "  - {}\n    Link: {}/threads/{}",
            res.title.bold(),
            DOMAIN,
            res.id
        );
    }
}

fn print_error_results(results: &Vec<ParsingResult>) {
    let iter = results.iter().filter(|r| r.error.is_some());
    if iter.clone().count() == 0 {
        return;
    }
    println!("\n{}:", "Error".bright_red());
    for res in iter {
        println!(
            "  - {}\n    Error: {}",
            format!("{}/threads/{}", DOMAIN, res.id).bold(),
            res.error.clone().unwrap_or_default().to_string(),
        );
    }
}

fn cache_results(results: &Vec<ParsingResult>, timestamp: i64) -> Result<()> {
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
            println!("{}: Offset should be positive.", "Error".red());
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
        println!("{}: No cached results found.", "Error".red());
        process::exit(0);
    }

    dirs_with_metadata.sort_by_key(|&(creation_time, _)| creation_time);
    dirs_with_metadata.reverse();

    let Some((_, entry)) = dirs_with_metadata.get(offset as usize) else {
        println!(
            "{}: Entry with offset {offset} does not exist.",
            "Error".red()
        );
        process::exit(0);
    };

    let file = entry.path().join("results.json");
    if !file.exists() {
        println!("{}: Results file is missing.", "Error".red());
        process::exit(0);
    }

    let file = fs::File::open(file).wrap_err("failed to open cached results file")?;
    let results: Vec<ParsingResult> =
        serde_json::from_reader(file).wrap_err("failed to deserialize cached results")?;

    print_check_results(&results);

    Ok(())
}

use std::{fs, path, process, time};
use std::collections::HashSet;

use color_eyre::eyre::{Result, WrapErr};
use color_eyre::owo_colors::OwoColorize;
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use ureq::{Agent, AgentBuilder};

use crate::parsing::{parse_thread, ParsingResult, Status};

pub fn check(file: &path::PathBuf) -> Result<()> {
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
        .user_agent(format!(
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

    print_check_results(results);

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

fn print_check_results(results: Vec<ParsingResult>) {
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
            "  - {}\n    Link: https://f95zone.to/threads/{}",
            res.title.bold(),
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
            format!("https://f95zone.to/threads/{}", res.id).bold(),
            res.error.clone().unwrap_or_default().to_string(),
        );
    }
}

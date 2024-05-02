use std::{fs, path};

use chrono::offset::Utc;
use colored::Colorize;

pub fn app_dir() -> path::PathBuf {
    let home = dirs::home_dir().expect("failed to locate home folder?");
    let app_dir = home.join("Documents").join(env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&app_dir).expect("failed to create app dir");
    app_dir
}

pub fn cache_dir() -> path::PathBuf {
    let app_dir = app_dir();
    let cache_dir = app_dir.join("cached_results");
    fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
    cache_dir
}

pub fn day_passed_since_last_check() -> bool {
    let now = Utc::now().timestamp();
    let past = get_check_timestamp();
    if (now - past) > 86400 {
        return true;
    }
    false
}

pub fn get_check_timestamp() -> i64 {
    let app_dir = app_dir();
    let file = app_dir.join("timestamp");
    if !file.exists() {
        return 0;
    }
    let string = fs::read_to_string(file).expect("failed to read timestamp");
    let Ok(stamp) = string.parse::<i64>() else {
        return 0;
    };
    stamp
}

pub fn save_check_timestamp(timestamp: i64) {
    let app_dir = app_dir();
    let file = app_dir.join("timestamp");
    fs::write(file, timestamp.to_string()).expect("failed to write a timestamp to a file");
}

pub fn error(msg: &str) {
    println!("{}: {}", "Error".red(), msg);
}

pub fn warning(msg: &str) {
    println!("{}: {}", "Warning".yellow(), msg);
}

pub fn success(msg: &str) {
    println!("{}: {}", "Success".green(), msg);
}

use std::{fs, path};

use chrono::offset::Utc;

pub fn app_dir() -> path::PathBuf {
    let home = dirs::home_dir().expect("failed to locate home folder?");
    let app_dir = home.join("Documents").join(env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&app_dir).expect("failed to create app dir");
    return app_dir;
}

pub fn cache_dir() -> path::PathBuf {
    let app_dir = app_dir();
    let cache_dir = app_dir.join("cached_results");
    fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
    return cache_dir;
}

pub fn day_passed_since_last_check() -> bool {
    let now = Utc::now().timestamp();
    let past = get_check_timestamp();
    if (now - past) > 86400 {
        return true;
    }
    return false;
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
    return stamp;
}

pub fn save_check_timestamp(timestamp: i64) {
    let app_dir = app_dir();
    let file = app_dir.join("timestamp");
    fs::write(file, timestamp.to_string()).expect("failed to write a timestamp to a file");
}

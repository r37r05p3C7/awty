use std::{fs, path, time};

pub fn app_dir() -> path::PathBuf {
    let home = dirs::home_dir().expect("failed to locate home folder?");
    let app_dir = home.join("Documents").join(env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&app_dir).expect("failed to create app dir");
    return app_dir;
}

pub fn day_passed_since_last_check() -> bool {
    let now = time::SystemTime::now();
    let past = time::UNIX_EPOCH + time::Duration::from_secs(get_check_timestamp());
    let elapsed = now.duration_since(past).expect("time went backwards");
    if elapsed.as_secs() > 86400 {
        return true;
    }
    return false;
}

pub fn get_check_timestamp() -> u64 {
    let app_dir = app_dir();
    let file = app_dir.join("timestamp");
    if !file.exists() {
        return 0;
    }
    let string = fs::read_to_string(file).expect("failed to read timestamp");
    let Ok(stamp) = string.parse::<u64>() else {
        return 0;
    };
    return stamp;
}

pub fn save_check_timestamp() {
    let app_dir = app_dir();
    let file = app_dir.join("timestamp");
    let now = time::SystemTime::now();
    let since_epoch = now
        .duration_since(time::UNIX_EPOCH)
        .expect("time went backwards");
    fs::write(file, since_epoch.as_secs().to_string())
        .expect("failed to write a timestamp to a file");
}

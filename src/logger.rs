use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use crate::env;
use once_cell::sync::Lazy;
use std::path::PathBuf;

static LOG_FILE: Lazy<PathBuf> = Lazy::new(|| {
    PathBuf::from(env::get("root_dir"))
        .join("logs")
        .join("log.txt")
});

pub fn log(msg: &str) {
    let line = format!(
        "[{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg
    );
    println!("{}", line);
    fs::create_dir_all("logs").ok();
    if let Ok(mut f) = OpenOptions::new()
        .append(true).create(true).open(&*LOG_FILE)
    {
        writeln!(f, "{}", line).ok();
    }
}
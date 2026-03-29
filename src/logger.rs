#![windows_subsystem = "windows"]
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;

const LOG_FILE: &str = "..\\intruder_alert_system\\logs\\log.txt";

pub fn log(msg: &str) {
    let line = format!(
        "[{}] {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        msg
    );
    println!("{}", line);
    fs::create_dir_all("..\\intruder_alert_system\\logs").ok();
    if let Ok(mut f) = OpenOptions::new()
        .append(true).create(true).open(LOG_FILE)
    {
        writeln!(f, "{}", line).ok();
    }
}
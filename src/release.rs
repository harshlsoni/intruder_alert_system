#![windows_subsystem = "windows"]
mod logger;

use std::net::TcpStream;
use std::io::Write;
use logger::log;

const PORT: u16 = 19876;

fn main() {
    log("=== PC unlocked ===");

    match TcpStream::connect(format!("127.0.0.1:{}", PORT)) {
        Ok(mut s) => {
            s.write_all(b"RELEASE").ok();
            log("Release signal sent to warm.exe");
        }
        Err(_) => {
            log("warm.exe was not running — nothing to release");
        }
    }
}
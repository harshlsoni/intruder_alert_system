

use security_cam::{db, logger::log};
use std::{thread, time::Duration, fs};
use security_cam::env;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use security_cam::config::load_env;
static TEMP_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PathBuf::from(env::get("root_dir"))
        .join("temp")
});
const INTERVAL_SECS: u64 = 10;

fn main() {
    load_env();
    log(" Retry worker started...");

    loop {
        process_temp_files();

        thread::sleep(Duration::from_secs(INTERVAL_SECS));
    }
}

fn process_temp_files() {
    let entries = match fs::read_dir(&*TEMP_DIR) {
        Ok(e) => e,
        Err(_) => {
            log(" Temp directory not found");
            return;
        }
    };

    for entry in entries {
        let path = match entry {
            Ok(e) => e.path(),
            Err(_) => continue,
        };

        if !path.is_file() {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();

        log(&format!(" Found file: {}", path_str));

        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                log(&format!(" Failed to read file: {:?}", e));
                continue;
            }
        };

        match db::save_attempt(&bytes) {
            Ok(id) => {
                log(&format!(" Recovered → saved to DB: {}", id));

                
                std::process::Command::new("mailer.exe")
                    .arg(id.to_string())
                    .spawn()
                    .ok();

                
                match fs::remove_file(&path) {
                    Ok(_) => log(" Temp file deleted"),
                    Err(e) => log(&format!(" Delete failed: {:?}", e)),
                }
            }

            Err(e) => {
                log(&format!(" Retry failed: {:?}", e));
            }
        }
    }
}
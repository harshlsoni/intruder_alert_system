
use security_cam::{db, logger::log};
use security_cam::config::load_env;


fn main() {
    
    load_env();

    let path = std::env::args().nth(1).expect("No file path");

    log(&format!(" Processing file: {}", path));

    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            log(&format!(" Failed to read file: {:?}", e));
            return;
        }
    };

    match db::save_attempt(&bytes) {
        Ok(id) => {
            log(&format!(" Saved to DB: {}", id));

            
            std::process::Command::new("mailer.exe")
                .arg(id.to_string())
                .spawn()
                .ok();

            
            match std::fs::remove_file(&path) {
                Ok(_) => log(" Temp file deleted"),
                Err(e) => log(&format!(" Delete failed: {:?}", e)),
            }
        }

        Err(e) => {
            log(&format!(" DB save failed: {:?}", e));
            log(" Keeping file for retry");
        }
    }
}
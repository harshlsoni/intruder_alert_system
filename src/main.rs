mod logger;
mod camera;
mod mailer;

use std::time::Instant;
use logger::log;

fn main() {
    let timer = Instant::now();
    log("=== Failed login detected ===");

    match camera::capture() {
        Some(filepath) => {
            log(&format!("Captured in {:.3}s", timer.elapsed().as_secs_f64()));
            mailer::send_alert(&filepath);
        }
        None => log("ERROR: Capture failed"),
    }

    log(&format!("Total time: {:.3}s", timer.elapsed().as_secs_f64()));
}
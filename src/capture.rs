#![windows_subsystem = "windows"]

mod logger;
mod mailer;

use std::net::TcpStream;
use std::io::{Write, Read};
use std::time::Instant;
use logger::log;

const PORT: u16 = 19876;

fn main() {
    let timer = Instant::now();
    log("=== Failed login detected ===");

    match TcpStream::connect(format!("127.0.0.1:{}", PORT)) {
        Ok(mut stream) => {
            // Send capture signal to warm.exe
            stream.write_all(b"CAPTURE").ok();

            // Read back the saved filepath
            let mut buf = vec![0u8; 512];
            let n       = stream.read(&mut buf).unwrap_or(0);
            let filepath = String::from_utf8_lossy(&buf[..n])
                            .trim().to_string();

            if filepath == "ERROR" || filepath.is_empty() {
                log("ERROR: Camera not ready or capture failed");
            } else {
                log(&format!(
                    "Captured in {:.3}s → {}",
                    timer.elapsed().as_secs_f64(),
                    filepath
                ));
                mailer::send_alert(&filepath);
            }
        }

        Err(_) => {
            // warm.exe not running — PC was not locked first
            // fallback: launch direct capture
            log("WARN: Camera not pre-warmed. Attempting direct capture...");
            direct_capture();
        }
    }

    log(&format!("Total: {:.3}s", timer.elapsed().as_secs_f64()));
}

fn direct_capture() {
    use escapi;
    use image::{ImageBuffer, Rgb};
    use image::codecs::jpeg::JpegEncoder;
    use std::fs;
    use chrono::Local;

    const WIDTH:    u32  = 640;
    const HEIGHT:   u32  = 480;
    const SAVE_DIR: &str = "intruder_alert_system\\captures";

    fs::create_dir_all(SAVE_DIR).ok();

    let mut cam = match escapi::init(0, WIDTH, HEIGHT, 30) {
        Ok(c)  => c,
        Err(e) => { log(&format!("ERROR: {:?}", e)); return; }
    };

    // warmup
    for _ in 0..2 { cam.capture().ok(); }

    let data = match cam.capture() {
        Ok(d)  => d,
        Err(e) => { log(&format!("ERROR: {:?}", e)); return; }
    };

    let mut rgb = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for chunk in data.chunks(4) {
        rgb.push(chunk[2]);
        rgb.push(chunk[1]);
        rgb.push(chunk[0]);
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filepath  = format!("{}\\intruder_{}.jpg", SAVE_DIR, timestamp);

    if let Some(img) = ImageBuffer::<Rgb<u8>, _>::from_raw(WIDTH, HEIGHT, rgb) {
        if let Ok(file) = fs::File::create(&filepath) {
            let mut enc = JpegEncoder::new_with_quality(
                std::io::BufWriter::new(file), 85
            );
            if enc.encode_image(&img).is_ok() {
                log(&format!("Photo saved (fallback): {}", filepath));
                mailer::send_alert(&filepath);
            }
        }
    }
}
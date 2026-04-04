#![windows_subsystem = "windows"]
use dotenv::from_path;
use security_cam::{logger::log, config};
use std::net::TcpStream;
use std::io::{Write, Read};
use std::time::Instant;
use security_cam::config::load;
use std::process::Command;
use std::path::PathBuf;
use chrono::Local;
use security_cam::env;
use once_cell::sync::Lazy;
use security_cam::config::load_env;

const PORT: u16 = 19876;

static TEMP_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PathBuf::from(env::get("root_dir"))
        .join("temp")
});



fn main() {
    if !config::is_enabled() {
        log("Tool is disabled — skipping capture.");
        return;
    }

    let timer = Instant::now();
    log("=== Failed login detected ===");
    
    load_env();

    // ── STEP 1: Get image bytes ───────────────────────────
    let img_bytes = match get_image_bytes() {
        Some(bytes) => bytes,
        None => {
            log(" Failed to obtain image");
            return;
        }
    };

    log(&format!(
        " Image ready ({} bytes) in {:.3}s",
        img_bytes.len(),
        timer.elapsed().as_secs_f64()
    ));


    let timestamp = Local::now().format("%Y%m%d_%H%M%S");

    let temp_path = format!(
        "{}\\{}.jpg",
        TEMP_DIR.display(),
        timestamp
    );

    std::fs::create_dir_all(&*TEMP_DIR).ok();

    match std::fs::write(&temp_path, &img_bytes) {
        Ok(_) => log(&format!(" Temp saved: {}", temp_path)),
        Err(e) => {
            log(&format!(" Temp save failed: {:?}", e));
            return;
        }
    }

    // ── STEP 2: Save to MongoDB and send mail ───────────────────────────

    Command::new("db_writer.exe")
    .arg(&temp_path)
    .spawn()
    .ok();


}


fn get_image_bytes() -> Option<Vec<u8>> {
    match TcpStream::connect(format!("127.0.0.1:{}", PORT)) {
        Ok(mut stream) => {
            log("📡 Connected to warm.exe");

            stream.write_all(b"CAPTURE").ok();

            let mut buf = vec![0u8; 1024 * 512];
            let n = stream.read(&mut buf).unwrap_or(0);

            if n == 0 {
                log(" No data received from warm.exe");
                return None;
            }

            let img_bytes = buf[..n].to_vec();

            log(&format!(" Received {} bytes from warm.exe", img_bytes.len()));

            Some(img_bytes)   
        }

        Err(_) => {
            log(" warm.exe not running → fallback capture");
            direct_capture()
        }
    }
}


fn direct_capture() -> Option<Vec<u8>> {
    use image::{ImageBuffer, Rgb};
    use image::codecs::jpeg::JpegEncoder;
    use std::io::{BufWriter, Cursor};

    const WIDTH:  u32 = 640;
    const HEIGHT: u32 = 480;

    log(" Opening camera directly...");

    let cam = match escapi::init(0, WIDTH, HEIGHT, 30) {
        Ok(c) => c,
        Err(e) => {
            log(&format!(" Camera open error: {:?}", e));
            return None;
        }
    };

    for _ in 0..2 { cam.capture().ok(); }

    let data = match cam.capture() {
        Ok(d) => d,
        Err(e) => {
            log(&format!(" Capture error: {:?}", e));
            return None;
        }
    };

    let mut rgb = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for chunk in data.chunks(4) {
        rgb.push(chunk[2]);
        rgb.push(chunk[1]);
        rgb.push(chunk[0]);
    }

    let img = ImageBuffer::<Rgb<u8>, _>::from_raw(WIDTH, HEIGHT, rgb)?;

    let mut jpeg_bytes = Vec::new();
    {
        let cursor = Cursor::new(&mut jpeg_bytes);
        let writer = BufWriter::new(cursor);
        let mut enc = JpegEncoder::new_with_quality(writer, 85);

        if let Err(e) = enc.encode_image(&img) {
            log(&format!(" JPEG encode error: {}", e));
            return None;
        }
    }

    log(&format!(" Direct capture: {} bytes", jpeg_bytes.len()));
    Some(jpeg_bytes)
}
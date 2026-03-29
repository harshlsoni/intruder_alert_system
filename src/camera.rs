use chrono::Local;
use std::fs;
use std::io::BufWriter;
use image::{ImageBuffer, Rgb};
use image::codecs::jpeg::JpegEncoder;
use crate::logger::log;

const SAVE_DIR: &str = "..\\intruder_alert_system\\captures";
const WIDTH:    u32  = 640;
const HEIGHT:   u32  = 480;

pub fn capture() -> Option<String> {
    fs::create_dir_all(SAVE_DIR).ok();

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filepath  = format!("{}\\intruder_{}.jpg", SAVE_DIR, timestamp);

    log("Opening camera...");

    // ── escapi v4 correct API ─────────────────────────────
    let cam = match escapi::init(0, WIDTH, HEIGHT, 30) {
        Ok(c)  => c,
        Err(e) => { log(&format!("ERROR opening camera: {:?}", e)); return None; }
    };

    // ── Single capture — no warmup needed ─────────────────
    let data = match cam.capture() {
        Ok(d)  => d,
        Err(e) => { log(&format!("ERROR capturing: {:?}", e)); return None; }
    };

    // ── escapi returns BGRA — convert to RGB ──────────────
    let mut rgb: Vec<u8> = Vec::with_capacity((WIDTH * HEIGHT * 3) as usize);
    for chunk in data.chunks(4) {
        rgb.push(chunk[2]); // R
        rgb.push(chunk[1]); // G
        rgb.push(chunk[0]); // B
    }

    // ── Build image buffer ────────────────────────────────
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        match ImageBuffer::from_raw(WIDTH, HEIGHT, rgb) {
            Some(b) => b,
            None    => { log("ERROR: Buffer build failed"); return None; }
        };

    // ── Save as JPEG (fast) ───────────────────────────────
    let file = match fs::File::create(&filepath) {
        Ok(f)  => f,
        Err(e) => { log(&format!("ERROR creating file: {}", e)); return None; }
    };

    let writer  = BufWriter::new(file);
    let mut enc = JpegEncoder::new_with_quality(writer, 85);

    if let Err(e) = enc.encode_image(&img) {
        log(&format!("ERROR encoding JPEG: {}", e));
        return None;
    }

    log(&format!("Photo saved: {}", filepath));
    Some(filepath)
}
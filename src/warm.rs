#![windows_subsystem = "windows"]
mod logger;

use std::net::TcpListener;
use std::io::{Read, Write};
use std::fs;
use image::{ImageBuffer, Rgb};
use image::codecs::jpeg::JpegEncoder;
use chrono::Local;
use logger::log;

const PORT:     u16  = 19876;
const WIDTH:    u32  = 640;
const HEIGHT:   u32  = 480;
const SAVE_DIR: &str = "intruder_alert_system\\captures";

fn main() {
    log("=== Lock screen detected ===");
    log("Warming up camera...");

    fs::create_dir_all(SAVE_DIR).ok();

    // Open camera
    let mut cam = match escapi::init(0, WIDTH, HEIGHT, 30) {
        Ok(c)  => c,
        Err(e) => {
            log(&format!("ERROR opening camera: {:?}", e));
            return;
        }
    };

    // Single warmup frame — gets camera ready
    cam.capture().ok();
    log("Camera warm. Waiting for commands...");

    // Bind TCP listener — waits for capture.exe or release.exe
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", PORT)) {
        Ok(l)  => l,
        Err(e) => {
            log(&format!("ERROR binding port: {}", e));
            return;
        }
    };

    // Wait for signals — uses 0% CPU while parked here
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut buf = [0u8; 32];
                let n       = s.read(&mut buf).unwrap_or(0);
                let signal  = String::from_utf8_lossy(&buf[..n])
                                .trim().to_string();

                match signal.as_str() {

                    "CAPTURE" => {
                        log("Capture signal received — grabbing frame...");

                        let data = match cam.capture() {
                            Ok(d)  => d,
                            Err(e) => {
                                log(&format!("ERROR grabbing frame: {:?}", e));
                                s.write_all(b"ERROR").ok();
                                continue;
                            }
                        };

                        // BGRA → RGB conversion
                        let mut rgb = Vec::with_capacity(
                            (WIDTH * HEIGHT * 3) as usize
                        );
                        for chunk in data.chunks(4) {
                            rgb.push(chunk[2]); // R
                            rgb.push(chunk[1]); // G
                            rgb.push(chunk[0]); // B
                        }

                        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
                        let filepath  = format!(
                            "{}\\intruder_{}.jpg",
                            SAVE_DIR, timestamp
                        );

                        let saved = match ImageBuffer::<Rgb<u8>, _>
                            ::from_raw(WIDTH, HEIGHT, rgb)
                        {
                            Some(img) => {
                                match fs::File::create(&filepath) {
                                    Ok(file) => {
                                        let mut enc = JpegEncoder::new_with_quality(
                                            std::io::BufWriter::new(file), 85
                                        );
                                        match enc.encode_image(&img) {
                                            Ok(_)  => {
                                                log(&format!("Photo saved: {}", filepath));
                                                true
                                            }
                                            Err(e) => {
                                                log(&format!("ERROR encoding: {}", e));
                                                false
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log(&format!("ERROR creating file: {}", e));
                                        false
                                    }
                                }
                            }
                            None => {
                                log("ERROR: Buffer build failed");
                                false
                            }
                        };

                        if saved {
                            s.write_all(filepath.as_bytes()).ok();
                        } else {
                            s.write_all(b"ERROR").ok();
                        }
                    }

                    "RELEASE" => {
                        log("Release signal — PC unlocked. Exiting.");
                        s.write_all(b"OK").ok();
                        break; 
                    }

                    _ => {
                        log(&format!("Unknown signal: {}", signal));
                    }
                }
            }
            Err(e) => {
                log(&format!("Connection error: {}", e));
                break;
            }
        }
    }

    log("Camera released. warm.exe exiting.");
}
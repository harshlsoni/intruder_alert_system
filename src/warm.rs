
use security_cam::logger;
use std::net::TcpListener;
use std::io::{Read, Write};
use image::{ImageBuffer, Rgb};
use image::codecs::jpeg::JpegEncoder;
use logger::log;
use security_cam::config;
use security_cam::config::load_env;

const PORT:     u16  = 19876;
const WIDTH:    u32  = 640;
const HEIGHT:   u32  = 480;

fn main() {
    if !config::is_enabled() {
        log("Tool is disabled — skipping capture.");
        return;
    }

    load_env();

    log("=== Lock screen detected ===");
    log("Warming up camera...");


    
    let  cam = match escapi::init(0, WIDTH, HEIGHT, 30) {
        Ok(c)  => c,
        Err(e) => {
            log(&format!("ERROR opening camera: {:?}", e));
            return;
        }
    };

    
    cam.capture().ok();
    log("Camera warm. Waiting for commands...");

    
    let listener = match TcpListener::bind(format!("127.0.0.1:{}", PORT)) {
        Ok(l)  => l,
        Err(e) => {
            log(&format!("ERROR binding port: {}", e));
            return;
        }
    };

    
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

                        
                        let mut rgb = Vec::with_capacity(
                            (WIDTH * HEIGHT * 3) as usize
                        );
                        for chunk in data.chunks(4) {
                            rgb.push(chunk[2]); 
                            rgb.push(chunk[1]); 
                            rgb.push(chunk[0]); 
                        }

                    
                        
                        let img = match ImageBuffer::<Rgb<u8>, _>::from_raw(WIDTH, HEIGHT, rgb) {
                        Some(i) => i,
                        None => {
                            log("ERROR: Buffer build failed");
                            s.write_all(b"ERROR").ok();
                            continue;
                        }
                    };

                   
                    let mut jpeg_bytes = Vec::new();
                    {
                        let mut enc = JpegEncoder::new_with_quality(&mut jpeg_bytes, 85);
                        if let Err(e) = enc.encode_image(&img) {
                            log(&format!("ERROR encoding: {}", e));
                            s.write_all(b"ERROR").ok();
                            continue;
                        }
                    }

                    log(&format!(" Captured image: {} bytes", jpeg_bytes.len()));

                    
                    s.write_all(&jpeg_bytes).ok();
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
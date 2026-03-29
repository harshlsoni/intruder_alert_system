use lettre::{
    Message,
    SmtpTransport,
    Transport,
    transport::smtp::authentication::Credentials,
    message::{header::ContentType, Attachment, MultiPart, SinglePart},
};
use std::fs;
use crate::logger::log;

// ── Fill these in ─────────────────────────────────────────
const SMTP_HOST:  &str = "smtp.gmail.com";
const SMTP_PORT:  u16  = 587;
const SMTP_USER:  &str = "";
const SMTP_PASS:  &str = "";    
const MAIL_FROM:  &str = "";
const MAIL_TO:    &str = ""; // where to receive alert
// ──────────────────────────────────────────────────────────

pub fn send_alert(filepath: &str) {
    log("Sending email alert...");

    // Read image bytes
    let img_bytes = match fs::read(filepath) {
        Ok(b)  => b,
        Err(e) => { log(&format!("ERROR reading image: {}", e)); return; }
    };

    // Build attachment
    let attachment = Attachment::new("intruder.png".to_string())
        .body(img_bytes, ContentType::parse("image/png").unwrap());

    // Build email
    let email = match Message::builder()
        .from(MAIL_FROM.parse().unwrap())
        .to(MAIL_TO.parse().unwrap())
        .subject("ALERT: Failed login attempt detected")
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(format!(
                            "A failed login attempt was detected on your laptop.\n\nPhoto is attached.\n\nFile: {}",
                            filepath
                        ))
                )
                .singlepart(attachment)
        ) {
            Ok(e)  => e,
            Err(e) => { log(&format!("ERROR building email: {}", e)); return; }
        };

    // Connect and send
    let creds = Credentials::new(SMTP_USER.to_string(), SMTP_PASS.to_string());

    let mailer = match SmtpTransport::starttls_relay(SMTP_HOST) {
        Ok(m)  => m.port(SMTP_PORT).credentials(creds).build(),
        Err(e) => { log(&format!("ERROR creating mailer: {}", e)); return; }
    };

    match mailer.send(&email) {
        Ok(_)  => log("Email sent successfully"),
        Err(e) => log(&format!("ERROR sending email: {}", e)),
    }
}
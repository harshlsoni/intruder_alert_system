
use lettre::{
    Message,
    SmtpTransport,
    Transport,
    transport::smtp::authentication::Credentials,
    message::{header::ContentType,Mailbox, Attachment, MultiPart, SinglePart},
};
use security_cam::config::load_env;
use security_cam::logger::log;
use security_cam::db;
use bson::oid::ObjectId;
use security_cam::env;
use std::env as std_env;
use once_cell::sync::Lazy;

// ── Fill these in ─────────────────────────────────────────
const SMTP_HOST:  &str = "smtp.gmail.com";
const SMTP_PORT:  u16  = 587;
static SMTP_USER: Lazy<String> = Lazy::new(|| env::get("SMTP_USER"));
static SMTP_PASS:  Lazy<String> =Lazy::new(|| env::get("SMTP_PASS"));
static MAIL_FROM:  Lazy<String> = Lazy::new(|| env::get("MAIL_FROM"));
static MAIL_TO:    Lazy<String> = Lazy::new(||env::get("MAIL_TO")); 
// ──────────────────────────────────────────────────────────

pub fn send_alert(id:ObjectId) {
    log(" [MAIL] Entered send_alert()");

    log(" [MAIL] Fetching image from DB...");

    

    let attempt = match db::get_attempt_by_id(id) {
        Ok(Some(a)) => a,
        Ok(None) => {
            log(" Attempt not found in DB");
            return;
        }
        Err(e) => {
            log(&format!(" DB fetch failed: {:?}", e));
            return;
        }
    };

    let timestamp = attempt.timestamp.clone();
    let bytes = attempt.image.bytes.clone();

    log(&format!(" [MAIL] Image size: {} bytes", bytes.len()));


    let from: Mailbox = match MAIL_FROM.parse() {
        Ok(m) => m,
        Err(e) => {
            log(&format!(" Invalid FROM email: {:?}", e));
            return;
        }
    };

    let to: Mailbox = match MAIL_TO.parse() {
        Ok(m) => m,
        Err(e) => {
            log(&format!(" Invalid TO email: {:?}", e));
            return;
        }
    };

    
    log(" [MAIL] Building attachment...");

    let content_type = match ContentType::parse("image/jpeg") {
        Ok(ct) => ct,
        Err(e) => {
            log(&format!(" ContentType parse failed: {:?}", e));
            return;
        }
    };

    let attachment = Attachment::new("intruder.jpg".to_string())
        .body(bytes.to_vec(), content_type);

    
    log(" [MAIL] Building email...");


    let body = format!(
        "SECURITY ALERT\n\n\
        A failed login attempt was detected on your system.\n\n\
        Time: {}\n\
        Device: Your Laptop\n\n\
        An image of the intruder is attached.\n\n\
        Stay safe.",
        timestamp
    );

    let email = match Message::builder()
        .from(from)
        .to(to)
        .subject("ALERT: Failed login attempt detected")
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(body)
                )
                .singlepart(attachment)
        ) {
        Ok(e) => {
            log(" [MAIL] Email built");
            e
        }
        Err(e) => {
            log(&format!(" Email build failed: {:?}", e));
            return;
        }
    };

    
    log(" [MAIL] Setting credentials...");

    let creds = Credentials::new(
        SMTP_USER.to_string(),
        SMTP_PASS.to_string()
    );

    log(" [MAIL] Creating SMTP transport...");

    let mailer = match SmtpTransport::starttls_relay(SMTP_HOST) {
        Ok(builder) => {
            log(" SMTP relay OK");
            builder.port(SMTP_PORT).credentials(creds).build()
        }
        Err(e) => {
            log(&format!(" SMTP creation failed: {:?}", e));
            return;
        }
    };

    log(" [MAIL] Sending email...");

    match mailer.send(&email) {
        Ok(res) => {
            log(&format!(" Email sent: {:?}", res));
        }
        Err(e) => {
            log(&format!(" Send failed: {:?}", e));
        }
    }

    log(" [MAIL] Finished send_alert()");
}



fn main() {
    load_env();
    let args: Vec<String> = std_env::args().collect();

    if args.len() < 2 {
        log(" No ID provided to mailer");
        return;
    }

    let id = match ObjectId::parse_str(&args[1]) {
        Ok(id) => id,
        Err(e) => {
            log(&format!(" Invalid ObjectId: {:?}", e));
            return;
        }
    };

    send_alert(id);
}
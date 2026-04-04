#![windows_subsystem = "windows"]
use dotenv::dotenv;
use security_cam::db;
use security_cam::config;
use security_cam::db::Attempt;
use mongodb::bson::oid::ObjectId;
use native_windows_gui as nwg;
use native_windows_derive as nwd;
use std::cell::RefCell;
use native_windows_gui::NativeUi;
use security_cam::logger::log;
use image::io::Reader as ImageReader;
use image::imageops::FilterType;
use image::codecs::jpeg::JpegEncoder;
use security_cam::config::load_env;
#[derive(Default, nwd::NwgUi)]

pub struct Dashboard {

    #[nwg_control(size: (800, 600), title: "Security Cam Dashboard", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [Dashboard::on_close])]
    window: nwg::Window,

    // ── STATUS ROW ─────────────────────────────
    #[nwg_control(text: "Protection: ON")]
    #[nwg_layout_item(layout: layout, row: 0, col: 0)]
    label_status: nwg::Label,

    #[nwg_control(text: "Disable")]
    #[nwg_layout_item(layout: layout, row: 0, col: 1)]
    #[nwg_events(OnButtonClick: [Dashboard::on_toggle])]
    btn_toggle: nwg::Button,

    // ── STATS ──────────────────────────────────
    #[nwg_control(text: "Total: 0 | Today: 0")]
    #[nwg_layout_item(layout: layout, row: 1, col: 0, col_span: 2)]
    label_total: nwg::Label,

    #[nwg_control(text: "Last attempt: never")]
    #[nwg_layout_item(layout: layout, row: 2, col: 0, col_span: 2)]
    label_last: nwg::Label,

    // ── MAIN CONTENT ───────────────────────────
    #[nwg_control()]
    #[nwg_layout_item(layout: layout, row: 3, col: 0)]
    #[nwg_events(OnListBoxSelect: [Dashboard::on_select])]
    list: nwg::ListBox<String>,

    #[nwg_control()]
    #[nwg_layout_item(layout: layout, row: 3, col: 1, stretch: true)]
    image_frame: nwg::ImageFrame,

    // ── IMAGE INFO ─────────────────────────────
    #[nwg_control(text: "")]
    #[nwg_layout_item(layout: layout, row: 4, col: 1)]
    label_timestamp: nwg::Label,

    // ── ACTION BUTTONS ─────────────────────────
    #[nwg_control(text: "Delete selected")]
    #[nwg_layout_item(layout: layout, row: 5, col: 0)]
    #[nwg_events(OnButtonClick: [Dashboard::on_delete])]
    btn_delete: nwg::Button,

    #[nwg_control(text: "Delete all")]
    #[nwg_layout_item(layout: layout, row: 5, col: 1)]
    #[nwg_events(OnButtonClick: [Dashboard::on_delete_all])]
    btn_delete_all: nwg::Button,

    #[nwg_control(text: "Refresh")]
    #[nwg_layout_item(layout: layout, row: 6, col: 1)]
    #[nwg_events(OnButtonClick: [Dashboard::on_refresh])]
    btn_refresh: nwg::Button,

    // ── Layout ─────────────────────────────────
    #[nwg_layout(parent: window, spacing: 5)]
    layout: nwg::GridLayout,

    // ── Internal state ─────────────────────────
    attempts: RefCell<Vec<Attempt>>,
    bitmap: RefCell<Option<nwg::Bitmap>>,
}

impl Dashboard {

    fn on_toggle(&self) {
        let enabled = config::is_enabled();
        config::set_enabled(!enabled);
        self.update_status();
    }

    fn update_status(&self) {
        if config::is_enabled() {
            self.label_status.set_text("Protection: ON");
            self.btn_toggle.set_text("Disable");
        } else {
            self.label_status.set_text("Protection: OFF");
            self.btn_toggle.set_text("Enable");
        }
    }

    fn on_refresh(&self) {
        self.refresh();
    }

    fn refresh(&self) {
        // Load stats
        let total = db::get_attempt_count().unwrap_or(0);
        let today = db::get_today_count().unwrap_or(0);
        self.label_total.set_text(&format!("Total: {}  |  Today: {}", total, today));

        // Load attempts
        let attempts = db::get_all_attempts().unwrap_or_default();

        let last = attempts.first()
            .map(|a| a.timestamp.clone())
            .unwrap_or_else(|| "never".to_string());
        self.label_last.set_text(&format!("Last attempt: {}", last));

        // Populate list
        self.list.clear();
        for a in &attempts {
            let tag = if a.emailed { " ✓" } else { "" };
            self.list.push(format!("{}{}", a.timestamp, tag));
        }

        *self.attempts.borrow_mut() = attempts;
        self.update_status();

        // Clear preview
        self.image_frame.set_bitmap(None);
        self.label_timestamp.set_text("");
        *self.bitmap.borrow_mut() = None;
    }

    fn on_select(&self) {
        let idx = match self.list.selection() {
            Some(i) => i,
            None    => return,
        };

        let attempt = {
            let attempts = self.attempts.borrow();

            match attempts.get(idx) {
                Some(a) => a.clone(), 
                None => {
                    log(" Invalid index selected");
                    return;
                }
            }
        };

        self.label_timestamp.set_text(&format!("Time: {}", attempt.timestamp));
        
        let bytes = &attempt.image.bytes.clone();

        
        let img = match ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .ok()
            .and_then(|r| r.decode().ok())
        {
            Some(i) => i,
            None => {
                println!(" Failed to decode image");
                return;
            }
        };

        let size = self.image_frame.size();
        let (frame_w, frame_h) = (size.0 as u32, size.1 as u32); 

        let resized = img.resize(frame_w, frame_h, FilterType::Triangle);

        let mut jpeg_bytes = Vec::new();
        {
            let mut encoder = JpegEncoder::new(&mut jpeg_bytes);
            if let Err(e) = encoder.encode_image(&resized) {
                println!(" JPEG encode error: {:?}", e);
                return;
            }
        }

        match nwg::Bitmap::from_bin(&jpeg_bytes) {
            Ok(bmp) => {
                *self.bitmap.borrow_mut() = Some(bmp);

                if let Some(ref bmp_ref) = *self.bitmap.borrow() {
                    self.image_frame.set_bitmap(Some(bmp_ref));
                }
            }
            Err(e) => {
                println!(" Bitmap error: {:?}", e);
            }
        }
        
    }


    fn on_delete(&self) {
        let idx = match self.list.selection() {
            Some(i) => i,
            None    => return,
        };

        let id: Option<ObjectId> = {
            let attempts = self.attempts.borrow();
            attempts.get(idx).and_then(|a| a.id)
        };

        if let Some(id) = id {
            db::delete_attempt(id).ok();
            self.refresh();
        }
    }

    fn on_delete_all(&self) {
        let params = nwg::MessageParams {
            title:   "Confirm",
            content: "Delete all attempts? This cannot be undone.",
            buttons: nwg::MessageButtons::YesNo,
            icons:   nwg::MessageIcons::Warning,
        };
        if nwg::message(&params) == nwg::MessageChoice::Yes {
            db::delete_all().ok();
            self.refresh();
        }
    }

    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }

}




fn main() {
    load_env();
    nwg::init().expect("Failed to init NWG");
    nwg::Font::set_global_family("Segoe UI").ok();

    let app = Dashboard::build_ui(Default::default())
        .expect("Failed to build UI");

    app.refresh();

    nwg::dispatch_thread_events();
}
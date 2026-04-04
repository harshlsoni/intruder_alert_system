#![windows_subsystem = "windows"]
#![allow(non_snake_case)]

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
use dotenv::from_path;
use nwg::NativeUi;
use security_cam::logger::log;
use security_cam::config;
use std::path::PathBuf;
use security_cam::env;
use once_cell::sync::Lazy;
use security_cam::config::load_env;
use security_cam::config::Config;
use security_cam::config::load;

static CONFIG: Lazy<Config> = Lazy::new(|| load());


#[derive(Default, nwd::NwgUi)]
pub struct TrayApp {

    // ── Hidden window ─────────────────────────────────────
    #[nwg_control(
        size: (0, 0),
        position: (0, 0),
        title: "SecurityCamController",
        flags: "WINDOW"
    )]
    #[nwg_events(OnWindowClose: [TrayApp::exit])]
    window: nwg::Window,

    // ── Icon resource ─────────────────────────────────────
    #[nwg_resource(
        source_file: Some("assets\\icon.ico")
    )]
    icon: nwg::Icon,

    // ── Tray notification ─────────────────────────────────
    #[nwg_control(
        icon: Some(&data.icon),
        tip: Some("Security Cam")
    )]
    #[nwg_events(
        MousePressLeftUp: [TrayApp::on_tray_click],
        OnContextMenu:    [TrayApp::show_menu]
    )]
    tray: nwg::TrayNotification,

    // ── Popup menu ────────────────────────────────────────
    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: "Open Dashboard")]
    #[nwg_events(OnMenuItemSelected: [TrayApp::open_dashboard])]
    menu_open: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Toggle Protection")]
    #[nwg_events(OnMenuItemSelected: [TrayApp::toggle])]
    menu_toggle: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "Exit")]
    #[nwg_events(OnMenuItemSelected: [TrayApp::exit])]
    menu_exit: nwg::MenuItem,
}

impl TrayApp {

    fn on_tray_click(&self) {
        self.show_menu();
    }

    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn toggle(&self) {
        let enabled = config::is_enabled();
        config::set_enabled(!enabled);
        
        let msg = if !enabled {
            "Protection ENABLED"
        } else {
            "Protection DISABLED"
        };

        log(&format!("Toggle: enabled={}", !enabled));
        nwg::simple_message("Security Cam", msg);
    }

    fn open_dashboard(&self) {
        

        static PATH: Lazy<PathBuf> = Lazy::new(|| {
            PathBuf::from(env::get("root_dir"))
                .join("target\\release\\dashboard.exe")
        });
        
        match std::process::Command::new(&*PATH).spawn() {
            Ok(_)  => log("Dashboard launched"),
            Err(e) => log(&format!("ERROR launching dashboard: {}", e)),
        }
    }

    fn exit(&self) {
        log("Controller exiting.");
        nwg::stop_thread_dispatch();
    }
}


fn main() {
    load_env();
    nwg::init().expect("Failed to init NWG");
    nwg::Font::set_global_family("Segoe UI").ok();

    let _app = TrayApp::build_ui(Default::default())
        .expect("Failed to build UI");

    nwg::dispatch_thread_events();
}
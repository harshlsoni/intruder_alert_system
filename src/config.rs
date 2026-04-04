use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::env;
use dotenv::from_path;
use std::env as std_env;
use std::path::PathBuf;
use once_cell::sync::Lazy;

static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    PathBuf::from(env::get("root_dir"))
        .join("config.toml")
});

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub enabled: bool

}

impl Default for Config {
    fn default() -> Self {
        Config { enabled: true }
    }
}

pub fn load() -> Config {
    let path = Path::new(&*CONFIG_PATH);

    if !path.exists() {
        let default = Config::default();
        save(&default);
        return default;
    }

    match fs::read_to_string(path) {
        Ok(s) => toml::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn load_env() {
    // Get path of current executable
    let exe_path = std_env::current_exe()
        .expect("Failed to get executable path");

    // Get directory where exe is located
    let exe_dir = exe_path
        .parent()
        .expect("Failed to get exe directory");

    // Build path to .env file
    let env_path: PathBuf = exe_dir.join(".env");

    // Load .env file
    from_path(&env_path).ok();

    // Optional debug (VERY useful while testing)
    println!("[ENV] Loaded from: {:?}", env_path);
}

pub fn save(config: &Config) {
    if let Some(parent) = Path::new(&*CONFIG_PATH).parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(s) = toml::to_string(config) {
        fs::write(&*CONFIG_PATH, s).ok();
    }
}

pub fn is_enabled() -> bool {
    let cfg = load();
    cfg.enabled
}

pub fn set_enabled(val: bool) {
    save(&Config { enabled: val });
}
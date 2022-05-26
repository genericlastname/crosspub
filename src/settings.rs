use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub site_name: String,
    pub site_url: String,
    pub default_index: bool,
    pub about: String,
    pub default_header: bool,
    pub default_footer: bool,
    pub header: String,
    pub footer: String,
    pub default_style: bool,
    pub css: String,
    pub html_root: String,
    pub gemini_root: String,
    pub post_dir: String,
}

#[derive(Deserialize)]
pub struct FMConfig {
    pub title: String,
    pub slug: String,
    pub date: String,
}

pub fn load_settings(path: &Path) -> Config {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            panic!("Couldn't read config file.");
        }
    };
    let config = match toml::from_str(&content) {
        Ok(c) => c,
        Err(_) => { panic!("Couldn't load valid config."); }
    };
    config
}

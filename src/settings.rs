use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub url: String,
    pub username: String,
    pub default_index: bool,
    pub about: String,
    pub html_template: String,
    pub gemini_template: String,
    pub default_style: bool,
    pub css: String,
    pub html_root: String,
    pub gemini_root: String,
    pub post_dir: String,
}

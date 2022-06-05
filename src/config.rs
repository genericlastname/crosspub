use serde::{Serialize, Deserialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub site: Site,
    pub homepage: Homepage,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Site {
    pub name: String,
    pub url: String,
    pub username: String,
    pub html_root: String,
    pub gemini_root: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Homepage {
    pub custom_homepage: bool,
    pub post_list: Option<bool>,
    pub use_about_page: Option<bool>,
}

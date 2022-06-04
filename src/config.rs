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
    pub posts_subdir: String,
    pub topics_subdir: String,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Homepage {
    pub custom_homepage: bool,
    pub list_posts_on_homepage: Option<bool>,
    pub use_about_page: Option<bool>,
}

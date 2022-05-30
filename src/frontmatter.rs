use serde::Deserialize;

#[derive(Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub slug: String,
    pub date: String,
}

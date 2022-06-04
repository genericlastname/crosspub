use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::exit;

use chrono::NaiveDate;
use serde::Serialize;
use toml;

use crate::frontmatter::Frontmatter;
use crate::gemtext::parse_gemtext;

#[derive(Clone, Debug, Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Post {
    pub title: String,
    pub filename: String,
    #[serde(with = "cp_date_format")]
    pub date: NaiveDate,
    pub html_content: String,
    pub gemini_content: String,
}

mod cp_date_format {
    use chrono::NaiveDate;
    use serde::{self, Serializer};

    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format("%Y-%m-%d"));
        serializer.serialize_str(&s)
    }
}

impl Default for Post {
    fn default() -> Post {
        Post {
            title: String::new(),
            filename: String::new(),
            date: NaiveDate::parse_from_str("1970-01-01", "%Y-%m-%d").unwrap(),
            html_content: String::new(),
            gemini_content: String::new(),
        }
    }
}

impl Post {
    pub fn from_source(source_path: PathBuf) -> Post {
        // Read from source .gmi file.
        let source = OpenOptions::new().read(true).open(&source_path);
        let source = match source {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Error: Could not open file {}",
                    &source_path.to_string_lossy());
                exit(1);
            },
        };
        let reader = BufReader::new(source);
        let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

        // Load frontmatter.
        let frontmatter: Frontmatter = match toml::from_str(&lines[1..=3].join("\n")) {
            Ok(fm) => fm,
            Err(_) => {
                eprintln!("Error: date formatted in {}", &source_path.to_string_lossy());
                exit(1);
            }
        };

        let mut post = Post::default();
        post.title = frontmatter.title;
        post.date = match NaiveDate::parse_from_str(&frontmatter.date, "%Y-%m-%d") {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Error: Date formatted incorrectly in {}",
                    &source_path.to_string_lossy());
                exit(1);
            }
        };
        post.filename = format!("{}_{}", post.date.format("%Y%m%d"), frontmatter.slug);

        // Generate content bodies for HTML and Gemini.
        let tokens = parse_gemtext(&lines[5..]);
        for token in tokens {
            post.html_content.push_str(&token.as_html())
        }
        post.gemini_content = lines[5..].join("\n");

        post
    }
}

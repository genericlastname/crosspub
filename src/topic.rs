use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::exit;

use serde::Serialize;
use toml::Value;

use crate::gemtext::parse_gemtext;

#[derive(Clone, Default, Debug, Serialize)]
pub struct Topic {
    pub title: String,
    pub filename: String,
    pub html_content: String,
    pub gemini_content: String,
}

impl Topic {
    pub fn from_source(source_path: PathBuf) -> Topic {
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
        let mut topic = Topic::default();
        topic.title = match lines[1].parse::<Value>() {
            Ok(v) => {
                let s = v["title"].to_string();
                let end = s.len() - 1;
                s[1..end].to_string()
            },
            Err(_) => {
                eprintln!("Could not parse frontmatter title.");
                exit(1);
            }
        };
        topic.filename = match lines[2].parse::<Value>() {
            Ok(v) => {
                let s = v["slug"].to_string();
                let end = s.len() - 1;
                s[1..end].to_string()
            },
            Err(_) => {
                eprintln!("Could not parse frontmatter slug.");
                exit(1);
            }
        };

        // Generate content bodies for HTML and Gemini.
        let tokens = parse_gemtext(&lines[5..]);
        for token in tokens {
            topic.html_content.push_str(&token.as_html())
        }
        topic.gemini_content = lines[4..].join("\n");

        topic
    }
}

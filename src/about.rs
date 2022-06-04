use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process::exit;

use serde::Serialize;

use crate::gemtext::parse_gemtext;

#[derive(Clone, Default, Debug, Serialize)]
pub struct About {
    pub html_content: String,
    pub gemini_content: String,
}

impl About {
    pub fn from_source(source_path: PathBuf) -> About {
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

        let mut about = About::default();

        // Generate content bodies for HTML and Gemini.
        let tokens = parse_gemtext(&lines);
        for token in tokens {
            about.html_content.push_str(&token.as_html())
        }
        about.gemini_content = lines.join("\n");

        about
    }
}

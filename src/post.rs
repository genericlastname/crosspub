use std::io::{BufRead, BufReader, Read, Write};
use std::fs::File;
use std::path::Path;

use chrono::NaiveDate;
use toml;

use crate::settings::{Config, FMConfig};
use crate::html::generate_html_from_tokens;
use crate::gemtext::{GemtextToken, parse_gemtext};

pub fn generate_filename(fm: &str) -> String {
    let config: FMConfig = match toml::from_str(fm) {
        Ok(c) => c,
        Err(_) => { panic!("Malformed front matter"); }
    };
    let date = match NaiveDate::parse_from_str(&config.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => { panic!("Date format is incorrect, please use YYYY-MM-DD"); }
    };

    let formatted_date = date.format("%Y%m%d");
    let filename = format!("{}_{}", formatted_date, &config.slug);
    filename
}

pub fn create_html_post_header(config: &Config) -> String {
    let header = String::new();
}
pub fn create_post(
    path: &Path,
    post_list: &mut Vec<String>,
    config: &Config
) {
    // open source .gmi file.
    let source = File::open(path).unwrap();
    let mut reader = BufReader::new(source);
    let fm_length = 5;

    let lines: Vec<String> = reader.by_ref().lines().collect::<Result<_, _>>().unwrap();
    let fm = format!("{}\n{}\n{}", lines[1], lines[2], lines[3]);
    let filename = generate_filename(&fm);

    let mut content = String::new();
    for line in reader.lines().skip(fm_length) {
        content.push_str(&line.unwrap());
    }

    let tokens: Vec<GemtextToken> = parse_gemtext(&content);

    // get public_html/posts directory from config.
    let html_path_string = format!("{}/{}/{}.html", config.html_root, config.post_dir, filename);
    let html_path = Path::new(&html_path_string);
    let html_target = File::create(html_path);

    // get public_gemini/posts directory from config.

    // write post.

    post_list.push(filename);
}

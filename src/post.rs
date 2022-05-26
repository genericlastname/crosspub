use std::io::{BufRead, BufReader, Read, Write};
use std::fs::File;
use std::path::Path;

use chrono::NaiveDate;
use toml;

use crate::settings::{Config, FMConfig};
use crate::html::{
    create_html_post_head_tag,
    generate_html_from_tokens,
};
use crate::gemtext::{GemtextToken, parse_gemtext};

fn generate_filename(fm_config: &FMConfig) -> String {
    let date = match NaiveDate::parse_from_str(&fm_config.date, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => { panic!("Date format is incorrect, please use YYYY-MM-DD"); }
    };

    let formatted_date = date.format("%Y%m%d");
    let filename = format!("{}_{}", formatted_date, &fm_config.slug);
    filename
}

fn create_html_post(tokens: &Vec<GemtextToken>, fm_config: &FMConfig, config: &Config) {
    let html_path_string = format!("{}/{}/{}.html",
        config.html_root,
        config.post_dir,
        generate_filename(fm_config));
    let html_path = Path::new(&html_path_string);
    let mut html_target = File::create(html_path).expect("Could not create post file");
    
    // Write <head>
    html_target.write_all(create_html_post_head_tag(fm_config, config).as_bytes()).unwrap();

    // Write header div
    let header = match config.default_header {
        true => {
            format!(
                r#"
                <div id="header">
                <p>{}</p>
                <nav>
                <h2>navigation</h2>
                <ul>
                <li><a href="{}">Home</a></li>
                <li><a href="{}/about.html">About Me</a></li>
                <li><a href="gemini://{}">Gemini version</a></li>
                </ul>
                </nav>
                </div>"#,
                &config.site_name,
                &config.site_url,
                &config.site_url,
                &config.site_url
            )
        },
        false => {
            std::fs::read_to_string(&config.header).expect("Could not open header file.")
        }
    };
    html_target.write_all(header.as_bytes()).unwrap();

    // Write content div
    html_target.write_all(generate_html_from_tokens(tokens).as_bytes()).unwrap();
}

pub fn create_post(
    path: &Path,
    post_list: &mut Vec<String>,
    config: &Config
) {
    // open source .gmi file.
    let source = File::open(path).expect("Could not find source gemini file.");
    let mut reader = BufReader::new(source);
    let fm_length = 5;

    let lines: Vec<String> = reader.by_ref().lines().collect::<Result<_, _>>().unwrap();
    let fm = format!("{}\n{}\n{}", lines[1], lines[2], lines[3]);
    let fm_config: FMConfig = match toml::from_str(&fm) {
        Ok(c) => c,
        Err(_) => { panic!("Malformed front matter"); }
    };

    let mut content = String::new();
    for line in reader.lines().skip(fm_length) {
        content.push_str(&line.unwrap());
    }

    let tokens: Vec<GemtextToken> = parse_gemtext(&content);

    // Write HTML post.
    create_html_post(&tokens, &fm_config, config);

    post_list.push(generate_filename(&fm_config));
}

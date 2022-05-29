use std::io::Read;
use std::io::Write as IoWrite;
use std::fmt::Write;
use std::fs::{OpenOptions, read_dir};

use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;
use tinytemplate::TinyTemplate;

use crate::post::Post;
use crate::settings::Config;

fn long_date_formatter(value: &Value, output: &mut String) -> tinytemplate::error::Result<()> {
    match value {
        Value::Null => Ok(()),
        Value::String(s) => {
            let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .expect("Date formatted incorrectly.");
            write!(output, "{}", date.format("%B %e, %Y"))?;
            Ok(())
        }
        _ => Err(tinytemplate::error::Error::GenericError {
            msg: "Incorrect date".to_string(),
        }),
    }
}

fn display_write_info(post: &Post, path: &str) {
    println!("Writing \"{}\" to {}", post.title, path);
}

#[derive(Serialize)]
struct Context {
    site: Config,
    post: Post,
}

#[derive(Debug, Default)]
pub struct CrossPub {
    config: Config,
    posts: Vec<Post>,

    // User options, controlled by command line params
    verbose: bool,
}

impl CrossPub {
    pub fn new(c: &Config) -> CrossPub {
        CrossPub {
            config: c.clone(),
            posts: Vec::new(),
            verbose: false,
        }
    }

    pub fn load_dir(&mut self, path_str: &str) {
        let dir = read_dir(path_str);
        let dir = match dir {
            Ok(d) => d,
            Err(_) => {
                panic!("{} is not a directory.", path_str);
            }
        };
        
        for entry in dir {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.extension() != Some(std::ffi::OsStr::new("gmi")) {
                continue;
            }

            let post = Post::from_source(entry.path());
            let post = match post {
                Ok(p) => p,
                Err(_) => {
                    panic!("Could not open file");
                }
            };

            self.posts.push(post);
        }
    }

    pub fn write_html_posts(&self) {
        let template_file = OpenOptions::new()
            .read(true)
            .open(&self.config.html_template);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                panic!("Could not open HTML template");
            }
        };
        let mut template_buffer = String::new();
        template_file.read_to_string(&mut template_buffer)
            .expect("Could not read from HTML template.");
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        tt.add_formatter("long_date_formatter", long_date_formatter);
        tt.add_template("html", &template_buffer)
            .expect("Could not add template.");

        for post in &self.posts {
            let context = Context {
                site: self.config.clone(),
                post: post.clone(),
            };
            let post_path_string = format!("{}/{}/{}.html",
                self.config.html_root,
                self.config.post_dir,
                post.filename);

            if self.verbose { display_write_info(&post, &post_path_string); }

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&post_path_string);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    panic!("Could not open {} for writing", &post_path_string);
                }
            };

            let rendered = tt.render("html", &context).unwrap();
            output.write_all(rendered.as_bytes())
                .expect("Could not write to file");
        }
    }

    pub fn write_gemini_posts(&self) {
        let template_file = OpenOptions::new()
            .read(true)
            .open(&self.config.gemini_template);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                panic!("Could not open Gemini template");
            }
        };
        let mut template_buffer = String::new();
        template_file.read_to_string(&mut template_buffer)
            .expect("Could not read from Gemini template.");
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        tt.add_formatter("long_date_formatter", long_date_formatter);
        tt.add_template("gemini", &template_buffer)
            .expect("Could not add template.");

        for post in &self.posts {
            let context = Context {
                site: self.config.clone(),
                post: post.clone(),
            };
            let post_path_string = format!("{}/{}/{}.gmi",
                self.config.gemini_root,
                self.config.post_dir,
                post.filename);

            if self.verbose { display_write_info(&post, &post_path_string); }

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&post_path_string);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    panic!("Could not open {} for writing", &post_path_string);
                }
            };

            let rendered = tt.render("gemini", &context).unwrap();
            output.write_all(rendered.as_bytes())
                .expect("Could not write to file");
        }
    }
}

use std::io::Read;
use std::io::Write as IoWrite;
use std::fmt::Write;
use std::fs::{OpenOptions, read_dir};
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use chrono::NaiveDate;
use serde::Serialize;
use serde_json::Value;
use tinytemplate::TinyTemplate;

use crate::post::Post;
use crate::config::Config;

#[derive(Clone, Default, Parser)]
#[clap(author = "hiroantag", version, about)]
/// A cross publishing site generator
pub struct Args {
    #[clap(short, long, parse(from_os_str))]
    pub config: Option<std::path::PathBuf>,
    #[clap(parse(from_os_str))]
    pub dir: Option<std::path::PathBuf>,
}

// Context for template generation.
#[derive(Serialize)]
struct Context {
    site: Config,
    post: Post,
}

pub struct CrossPub {
    config: Config,
    posts: Vec<Post>,
}

impl CrossPub {
    pub fn new(c: &Config, a: &Args) -> CrossPub {
        let mut cp = CrossPub {
            config: c.clone(),
            posts: Vec::new(),
        };
        if let Some(d) = &a.dir {
            cp.load_dir(d.to_path_buf());
        } else {
            cp.load_dir(PathBuf::from("."));
        }

        cp
    }

    pub fn write_posts(&self) {
        self.write_html_posts();
        self.write_gemini_posts();
    }

    fn load_dir(&mut self, path: PathBuf) {
        let dir = read_dir(path);
        let dir = match dir {
            Ok(d) => d,
            Err(_) => {
                eprintln!("Error: Given path is not a directory.");
                exit(1);
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
                    // These unwraps are FINE, there's no way that the given path
                    // could be invalid at this point.
                    eprintln!("Error: Could not open file {}",
                        entry.path().file_name().unwrap().to_str().unwrap());
                    exit(1);
                }
            };

            self.posts.push(post);
        }
    }

    fn write_html_posts(&self) {
        // Open post template
        let template_file;
        let post_template_path: PathBuf;
        if self.config.templates.custom_templates {
            post_template_path = [
                self.config.templates.custom_html_path.as_ref().unwrap(),
                "post.html"
            ].iter().collect();
        } else {
            post_template_path = PathBuf::from("/usr/share/crosspub/templates/post.html");
        }
        template_file = OpenOptions::new()
            .read(true)
            .open(post_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open HTML template");
                exit(1);
            }
        };

        // Read template to String and load into parser.
        let mut template_buffer = String::new();
        match template_file.read_to_string(&mut template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from HTML template");
                exit(1)
            }
        }
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        tt.add_formatter("long_date_formatter", long_date_formatter);
        match tt.add_template("html", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not parse HTML post template file");
                exit(1)
            }
        }

        // Generate posts.
        for post in &self.posts {
            let context = Context {
                site: self.config.clone(),
                post: post.clone(),
            };
            let post_path: PathBuf = [
                &self.config.site.html_root,
                &self.config.site.posts_subdir,
                &post.filename
            ].iter().collect();

            display_write_info(&post, &post_path);

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&post_path);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    eprintln!("Error: Could not open {} for writing", &post_path.to_str().unwrap());
                    exit(1);
                }
            };

            // This unwrap is fine, render can only fail given an incorrect
            // template name.
            let rendered = tt.render("html", &context).unwrap();
            match output.write_all(rendered.as_bytes()) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Error: Could not write to {}", &post_path.to_str().unwrap());
                    exit(1)
                }
            }
        }
    }

    fn write_gemini_posts(&self) {
        // Open post template
        let template_file;
        let post_template_path: PathBuf;
        if self.config.templates.custom_templates {
            post_template_path = [
                self.config.templates.custom_gemini_path.as_ref().unwrap(),
                "post.gmi"
            ].iter().collect();
        } else {
            post_template_path = PathBuf::from("/usr/share/crosspub/templates/post.gmi");
        }
        template_file = OpenOptions::new()
            .read(true)
            .open(post_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open gemini template");
                exit(1);
            }
        };


        // Read template to String and load into parser.
        let mut template_buffer = String::new();
        match template_file.read_to_string(&mut template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from gemini template");
                exit(1)
            }
        }

        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        tt.add_formatter("long_date_formatter", long_date_formatter);
        match tt.add_template("gemini", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not parse gemini post template file");
                exit(1)
            }
        }

        // Generate posts.
        for post in &self.posts {
            let context = Context {
                site: self.config.clone(),
                post: post.clone(),
            };
            let post_path: PathBuf = [
                &self.config.site.gemini_root,
                &self.config.site.posts_subdir,
                &post.filename
            ].iter().collect();

            display_write_info(&post, &post_path);

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&post_path);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    eprintln!("Error: Could not open {} for writing", &post_path.to_str().unwrap());
                    exit(1);
                }
            };

            let rendered = tt.render("gemini", &context).unwrap();
            match output.write_all(rendered.as_bytes()) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Error: Could not write to {}", post_path.to_str().unwrap());
                    exit(1)
                }
            }
        }
    }
}

fn long_date_formatter(value: &Value, output: &mut String) -> tinytemplate::error::Result<()> {
    match value {
        Value::Null => Ok(()),
        Value::String(s) => {
            let date = NaiveDate::parse_from_str(&s, "%Y-%m-%d");
            let date = match date {
                Ok(d) => d,
                Err(_) => {
                    eprintln!(r#"
                Error: Date formatted incorrectly in TOML header
                Try:
                    date = "YYYY-MM-DD"
                "#);
                    exit(1);
                }
            };
            write!(output, "{}", date.format("%B %e, %Y"))?;
            Ok(())
        }
        _ => Err(tinytemplate::error::Error::GenericError {
            msg: "Incorrect date".to_string(),
        }),
    }
}

fn display_write_info(post: &Post, path: &PathBuf) {
    println!("Writing \"{}\" to {}", post.title, path.to_string_lossy());
}


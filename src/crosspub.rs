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
use crate::topic::Topic;
use crate::config::{Config, Site};

#[derive(Clone, Default, Parser)]
#[clap(author = "hiroantag", version, about)]
/// A cross publishing site generator
pub struct Args {
    #[clap(short, long, parse(from_os_str))]
    pub config: Option<std::path::PathBuf>,
    #[clap(parse(from_os_str))]
    pub dir: Option<std::path::PathBuf>,
}

// Contexts for template generation.
#[derive(Serialize)]
struct PostContext {
    site: Site,
    post: Post,
}

#[derive(Serialize)]
struct TopicContext {
    site: Site,
    topic: Topic,
}

#[derive(Serialize)]
struct IndexContext {
    site: Site,
    posts: Vec<Post>,
    topics: Vec<Topic>,
    has_topics: bool,
}

pub struct CrossPub {
    config: Config,
    posts: Vec<Post>,
    topics: Vec<Topic>,
}

impl CrossPub {
    pub fn new(c: &Config, a: &Args) -> CrossPub {
        let mut cp = CrossPub {
            config: c.clone(),
            posts: Vec::new(),
            topics: Vec::new(),
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

    pub fn write_topics(&self) {
        self.write_html_topics();
        self.write_gemini_topics();
    }
    
    pub fn generate_index(&self) {
        self.generate_index_html();
        self.generate_index_gmi();
    }

    fn generate_index_html(&self) {
        // Open index template
        let template_file;
        let index_template_path: PathBuf;
        if self.config.templates.custom_templates {
            index_template_path = [
                self.config.templates.custom_html_path.as_ref().unwrap(),
                "index.html"
            ].iter().collect();
        } else {
            index_template_path = PathBuf::from("/usr/share/crosspub/templates/html/index.html");
        }
        template_file = OpenOptions::new()
            .read(true)
            .open(index_template_path);
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
        match tt.add_template("html", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error could not parse HTML index template file");
                exit(1);
            }
        }

        let has_topics = !self.topics.is_empty();

        let context = IndexContext {
            site: self.config.site.clone(),
            posts: self.posts.clone(),
            topics: self.topics.clone(),
            has_topics
        };

        println!("Writing index.html");

        let index_path: PathBuf = [
            &self.config.site.html_root,
            "index.html",
        ].iter().collect();

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&index_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &index_path.to_string_lossy());
                exit(1);
            }
        };

        let rendered = tt.render("html", &context).unwrap();
        match output.write_all(rendered.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: Could not write to {}", &index_path.to_string_lossy());
                exit(1);
            }
        }
    }

    fn generate_index_gmi(&self) {
        // Open index template
        let template_file;
        let index_template_path: PathBuf;
        if self.config.templates.custom_templates {
            index_template_path = [
                self.config.templates.custom_gemini_path.as_ref().unwrap(),
                "index.gmi"
            ].iter().collect();
        } else {
            index_template_path = PathBuf::from("/usr/share/crosspub/templates/gemini/index.gmi");
        }
        template_file = OpenOptions::new()
            .read(true)
            .open(index_template_path);
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
        match tt.add_template("gemini", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error could not parse gemini index template file");
                exit(1);
            }
        }

        let has_topics = !self.topics.is_empty();

        let context = IndexContext {
            site: self.config.site.clone(),
            posts: self.posts.clone(),
            topics: self.topics.clone(),
            has_topics
        };

        println!("Writing index.gmi");

        let index_path: PathBuf = [
            &self.config.site.gemini_root,
            "index.gmi",
        ].iter().collect();

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&index_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &index_path.to_string_lossy());
                exit(1);
            }
        };

        let rendered = tt.render("gemini", &context).unwrap();
        match output.write_all(rendered.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: Could not write to {}", &index_path.to_string_lossy());
                exit(1);
            }
        }
    }

    fn load_dir(&mut self, path: PathBuf) {
        match read_dir(&path) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("Error: Given path is not a directory.");
                exit(1);
            }
        };
        let posts_path: PathBuf = [&path.to_str().unwrap(), "posts"].iter().collect();
        let posts_dir = match read_dir(posts_path) {
            Ok(pd) => pd,
            Err(_) => {
                eprintln!("Error: No posts/ directory.");
                exit(1);
            }
        };
        let topics_path: PathBuf = [&path.to_str().unwrap(), "topics"].iter().collect();
        let topics_dir = match read_dir(topics_path) {
            Ok(td) => td,
            Err(_) => {
                eprintln!("Error: No topics/ directory.");
                exit(1);
            }
        };
        
        for entry in posts_dir {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.extension() != Some(std::ffi::OsStr::new("gmi")) {
                continue;
            }

            let post = Post::from_source(entry.path());
            self.posts.push(post);
        }

        for entry in topics_dir {
            let entry = entry.unwrap();
            let t = entry.path();
            if t.extension() != Some(std::ffi::OsStr::new("gmi")) {
                continue;
            }

            let topic = Topic::from_source(entry.path());
            self.topics.push(topic);
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
            post_template_path = PathBuf::from("/usr/share/crosspub/templates/html/post.html");
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
            let context = PostContext {
                site: self.config.site.clone(),
                post: post.clone(),
            };
            let mut post_path: PathBuf = [
                &self.config.site.html_root,
                &self.config.site.posts_subdir,
                &post.filename,
            ].iter().collect();
            post_path.set_extension("html");

            println!("Writing \"{}\" to {}", &post.title, &post_path.to_string_lossy());

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&post_path);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    eprintln!("Error: Could not open {} for writing", &post_path.to_string_lossy());
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
                    exit(1);
                }
            }
        }
    }

    fn write_html_topics(&self) {
        // Open topic template
        let template_file;
        let topic_template_path: PathBuf;
        if self.config.templates.custom_templates {
            topic_template_path = [
                self.config.templates.custom_html_path.as_ref().unwrap(),
                "topic.html"
            ].iter().collect();
        } else {
            topic_template_path = PathBuf::from("/usr/share/crosspub/templates/html/topic.html");
        }
        template_file = OpenOptions::new()
            .read(true)
            .open(topic_template_path);
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
        match tt.add_template("html", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not parse HTML topic template file");
                exit(1)
            }
        }

        // Generate topics.
        for topic in &self.topics {
            let context = TopicContext {
                site: self.config.site.clone(),
                topic: topic.clone(),
            };
            let mut topic_path: PathBuf = [
                &self.config.site.html_root,
                &topic.filename
            ].iter().collect();
            topic_path.set_extension("html");

            println!("Writing \"{}\" to {}", &topic.title, &topic_path.to_str().unwrap());

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&topic_path);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    eprintln!("Error: Could not open {} for writing", &topic_path.to_str().unwrap());
                    exit(1);
                }
            };

            // This unwrap is fine, render can only fail given an incorrect
            // template name.
            let rendered = tt.render("html", &context).unwrap();
            match output.write_all(rendered.as_bytes()) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Error: Could not write to {}", &topic_path.to_str().unwrap());
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
            post_template_path = PathBuf::from("/usr/share/crosspub/templates/gemini/post.gmi");
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
            let context = PostContext {
                site: self.config.site.clone(),
                post: post.clone(),
            };
            let mut post_path: PathBuf = [
                &self.config.site.gemini_root,
                &self.config.site.posts_subdir,
                &post.filename
            ].iter().collect();
            post_path.set_extension("gmi");

            println!("Writing \"{}\" to {}", &post.title, &post_path.to_str().unwrap());

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

    fn write_gemini_topics(&self) {
        // Open topic template
        let template_file;
        let topic_template_path: PathBuf;
        if self.config.templates.custom_templates {
            topic_template_path = [
                self.config.templates.custom_gemini_path.as_ref().unwrap(),
                "topic.gmi"
            ].iter().collect();
        } else {
            topic_template_path = PathBuf::from("/usr/share/crosspub/templates/gemini/topic.gmi");
        }
        template_file = OpenOptions::new()
            .read(true)
            .open(topic_template_path);
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
        match tt.add_template("gemini", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not parse gemini topic template file");
                exit(1)
            }
        }

        // Generate topics.
        for topic in &self.topics {
            let context = TopicContext {
                site: self.config.site.clone(),
                topic: topic.clone(),
            };
            let mut topic_path: PathBuf = [
                &self.config.site.gemini_root,
                &topic.filename
            ].iter().collect();
            topic_path.set_extension("gmi");

            println!("Writing \"{}\" to {}", &topic.title, &topic_path.to_str().unwrap());

            let output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&topic_path);
            let mut output = match output {
                Ok(o) => o,
                Err(_) => {
                    eprintln!("Error: Could not open {} for writing", &topic_path.to_str().unwrap());
                    exit(1);
                }
            };

            let rendered = tt.render("gemini", &context).unwrap();
            match output.write_all(rendered.as_bytes()) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Error: Could not write to {}", topic_path.to_str().unwrap());
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

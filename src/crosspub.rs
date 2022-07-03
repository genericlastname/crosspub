use std::io::Read;
use std::io::Write as IoWrite;
use std::fmt::Write;
use std::fs::{self, OpenOptions, read_dir};
use std::path::PathBuf;
use std::process::exit;

use clap::Parser;
use chrono::{
    DateTime,
    offset::{Local, TimeZone},
    NaiveDate,
};
use serde_json::Value;
use tinytemplate::TinyTemplate;

use crate::about::About;
use crate::contexts::*;
use crate::post::Post;
use crate::topic::Topic;
use crate::config::Config;

#[derive(Clone, Default, Parser)]
#[clap(author = "hiroantag", version, about)]
/// A cross publishing site generator
pub struct Args {
    /// Path to config file
    #[clap(short, long, parse(from_os_str))]
    pub config: Option<std::path::PathBuf>,

    /// Path to directory with crosspub files. Defaults to PWD.
    #[clap(parse(from_os_str))]
    pub dir: Option<std::path::PathBuf>,

    /// Initialize a directory for crosspub
    #[clap(long)]
    pub init: bool,
}

pub struct CrossPub {
    config: Config,
    latest_post: Post,
    posts: Vec<Post>,
    topics: Vec<Topic>,
    about: About,
    xdg_dirs: xdg::BaseDirectories,
    post_listing: bool,
    has_about: bool,
}

impl CrossPub {
    pub fn new(c: &Config, a: &Args) -> CrossPub {
        let mut cp = CrossPub {
            config: c.clone(),
            latest_post: Post::default(),
            posts: Vec::new(),
            topics: Vec::new(),
            about: About::default(),
            xdg_dirs: xdg::BaseDirectories::with_prefix("crosspub").unwrap(),
            post_listing: false,
            has_about: false,
        };
        
        if let Some(d) = &a.dir {
            cp.load_dir(d.to_path_buf());
        } else {
            cp.load_dir(PathBuf::from("."));
        }

        if cp.posts.is_empty() {
            println!("No posts found.");
            exit(0);
        }

        if let Some(pl) = c.homepage.post_list {
            cp.post_listing = pl;
        }

        if let Some(a) = c.homepage.use_about_page {
            cp.has_about = a;
        }

        cp.latest_post = cp.posts[0].clone();

        if cp.has_about {
            let about_source_path = match cp.xdg_dirs.find_data_file("about.gmi") {
                Some(a) => a,
                _ => {
                    eprintln!("Error: Could not find about.gmi file in ~/.local/share/crosspub");
                    exit(1);
                }
            };
            cp.about = About::from_source(about_source_path);
        }

        cp
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
        self.posts.sort_by(|a, b| b.date.partial_cmp(&a.date).unwrap());

        for entry in topics_dir {
            let entry = entry.unwrap();
            let t = entry.path();
            if t.extension() != Some(std::ffi::OsStr::new("gmi")) {
                continue;
            }

            let topic = Topic::from_source(entry.path());
            self.topics.push(topic);
        }
        self.topics.sort_by(|a, b| a.title.partial_cmp(&b.title).unwrap());
    }

    pub fn write(&self) {
        self.write_html_posts();
        self.write_gemini_posts();
        self.write_html_topics();
        self.write_gemini_topics();
        self.generate_index_html();
        self.generate_index_gmi();
        self.copy_css();
        self.generate_html_atom_feed();
        self.generate_gemini_atom_feed();

        if self.has_about {
            self.generate_about_html();
            self.generate_about_gmi();
        }

        if self.post_listing {
            self.generate_post_listing_html();
            self.generate_post_listing_gmi();
        }
    }

    fn generate_index_html(&self) {
        // Open index template
        let template_file;
        let index_template_path = self.xdg_dirs.find_data_file("templates/html/index.html");
        let index_template_path = match index_template_path {
            Some(p) => p,
            _ => {
                eprintln!("Error: Could not find HTML index template.");
                exit(1);
            }
        };

        template_file = OpenOptions::new()
            .read(true)
            .open(index_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open HTML index template");
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
                eprintln!("Error Could not parse HTML index template file");
                exit(1);
            }
        }

        let context = IndexContext {
            site: self.config.site.clone(),
            latest_post: self.posts[0].clone(),
            posts: self.posts.clone(),
            topics: self.topics.clone(),
            has_topics: !self.topics.is_empty(),
            has_about: self.has_about,
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

    fn generate_post_listing_html(&self) {
        // Open post listing template
        let template_file;
        let postlist_template_path: PathBuf = [
            self.xdg_dirs.get_data_home(),
            PathBuf::from("templates/html/postlist.html"),
        ].iter().collect();

        template_file = OpenOptions::new()
            .read(true)
            .open(postlist_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open HTML postlist template");
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
                eprintln!("Error Could not parse HTML postlist template file");
                exit(1);
            }
        }

        let has_topics = !self.topics.is_empty();

        let context = IndexContext {
            site: self.config.site.clone(),
            latest_post: self.posts[0].clone(),
            posts: self.posts.clone(),
            topics: self.topics.clone(),
            has_topics,
            has_about: self.has_about,
        };

        println!("Writing postlist.html");

        let postlist_path: PathBuf = [
            &self.config.site.html_root,
            "posts",
            "posts.html",
        ].iter().collect();

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&postlist_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &postlist_path.to_string_lossy());
                exit(1);
            }
        };

        let rendered = tt.render("html", &context).unwrap();
        match output.write_all(rendered.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: Could not write to {}", &postlist_path.to_string_lossy());
                exit(1);
            }
        }
    }

    fn generate_post_listing_gmi(&self) {
        // Open post listing template
        let template_file;
        let postlist_template_path: PathBuf = [
            self.xdg_dirs.get_data_home(),
            PathBuf::from("templates/gemini/postlist.gmi"),
        ].iter().collect();

        template_file = OpenOptions::new()
            .read(true)
            .open(postlist_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open Gemini postlist template");
                exit(1);
            }
        };
        // Read template to String and load into parser.
        let mut template_buffer = String::new();
        match template_file.read_to_string(&mut template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from Gemini template");
                exit(1)
            }
        }
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        match tt.add_template("gemini", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error Could not parse Gemini postlist template file");
                exit(1);
            }
        }

        let has_topics = !self.topics.is_empty();

        let context = IndexContext {
            site: self.config.site.clone(),
            latest_post: self.posts[0].clone(),
            posts: self.posts.clone(),
            topics: self.topics.clone(),
            has_topics,
            has_about: self.has_about,
        };

        println!("Writing postlist.gmi");

        let postlist_path: PathBuf = [
            &self.config.site.gemini_root,
            "posts",
            "posts.gmi",
        ].iter().collect();

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&postlist_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &postlist_path.to_string_lossy());
                exit(1);
            }
        };

        let rendered = tt.render("gemini", &context).unwrap();
        match output.write_all(rendered.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: Could not write to {}", &postlist_path.to_string_lossy());
                exit(1);
            }
        }
    }

    fn generate_index_gmi(&self) {
        // Open index template
        let template_file;
        let index_template_path = self.xdg_dirs.find_data_file("templates/gemini/index.gmi");
        let index_template_path = match index_template_path {
            Some(p) => p,
            _ => {
                eprintln!("Error: Could not find Gemini index template.");
                exit(1);
            }
        };

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
            latest_post: self.posts[0].clone(),
            posts: self.posts.clone(),
            topics: self.topics.clone(),
            has_topics,
            has_about: self.has_about,
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

    fn copy_css(&self) {
        let css_source_path = match self.xdg_dirs.find_data_file("templates/html/style.css") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find source CSS file.");
                exit(1);
            }
        };

        let css_dir_path: PathBuf = [
            &self.config.site.html_root,
            "css",
        ].iter().collect();
        if !css_dir_path.exists() {
            match fs::create_dir(&css_dir_path) {
                Ok(_) => {},
                Err(_) => {
                    eprintln!("Error: Could not create directory at {}",
                        &css_dir_path.to_string_lossy());
                    exit(1);
                }
            }
        }
        
        let css_dest_path: PathBuf = [
            &css_dir_path.to_string_lossy(),
            "style.css",
        ].iter().collect();
        match fs::copy(css_source_path, css_dest_path) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not copy CSS file");
                exit(1);
            }
        }
    }

    fn generate_about_html(&self) {
        let about_template_path = match self.xdg_dirs.find_data_file("templates/html/about.html") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find HTML post template.");
                exit(1);
            }
        };
        let template_file = OpenOptions::new()
            .read(true)
            .open(about_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open HTML about template");
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
                eprintln!("Error: Could not parse HTML about template file");
                exit(1)
            }
        }

        let context = AboutContext {
            site: self.config.site.clone(),
            about: self.about.clone(),
            has_about: self.has_about,
        };
        let about_path: PathBuf = [
            &self.config.site.html_root,
            "about.html"
        ].iter().collect();

        println!("Writing about.html to {}", &about_path.to_string_lossy());

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&about_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &about_path.to_string_lossy());
                exit(1);
            }
        };
        let rendered = tt.render("html", &context).unwrap();
        match output.write_all(rendered.as_bytes()) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not write to {}", &about_path.to_str().unwrap());
                exit(1);
            }
        }
    }

    fn generate_about_gmi(&self) {
        let about_template_path = match self.xdg_dirs.find_data_file("templates/gemini/about.gmi") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find Gemini post template.");
                exit(1);
            }
        };
        let template_file = OpenOptions::new()
            .read(true)
            .open(about_template_path);
        let mut template_file = match template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open Gemini about template");
                exit(1);
            }
        };

        // Read template to String and load into parser.
        let mut template_buffer = String::new();
        match template_file.read_to_string(&mut template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from Gemini template");
                exit(1)
            }
        }
        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        match tt.add_template("gemini", &template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not parse Gemini about template file");
                exit(1)
            }
        }

        let context = AboutContext {
            site: self.config.site.clone(),
            about: self.about.clone(),
            has_about: self.has_about,
        };
        let about_path: PathBuf = [
            &self.config.site.gemini_root,
            "about.gmi"
        ].iter().collect();

        println!("Writing about.gmi to {}", &about_path.to_string_lossy());

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&about_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &about_path.to_string_lossy());
                exit(1);
            }
        };
        let rendered = tt.render("gemini", &context).unwrap();
        match output.write_all(rendered.as_bytes()) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not write to {}", &about_path.to_str().unwrap());
                exit(1);
            }
        }
    }

    fn write_html_posts(&self) {
        // Open post template
        let template_file;
        let post_template_path = match self.xdg_dirs.find_data_file("templates/html/post.html") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find HTML post template.");
                exit(1);
            }
        };
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
                has_about: self.has_about,
            };
            let mut post_path: PathBuf = [
                &self.config.site.html_root,
                "posts",
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
        let topic_template_path = match self.xdg_dirs.find_data_file("templates/html/topic.html") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find HTML topic template.");
                exit(1);
            }
        };
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
                has_about: self.has_about,
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
        let post_template_path = match self.xdg_dirs.find_data_file("templates/gemini/post.gmi") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find Gemini post template.");
                exit(1);
            }
        };
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
                has_about: self.has_about,
            };
            let mut post_path: PathBuf = [
                &self.config.site.gemini_root,
                "posts",
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
        let topic_template_path = match self.xdg_dirs.find_data_file("templates/gemini/topic.gmi") {
            Some(t) => t,
            _ => {
                eprintln!("Error: Could not find Gemini topic template.");
                exit(1);
            }
        };
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
                has_about: self.has_about,
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

    fn generate_gemini_atom_feed(&self) {
        let feed_template_file;
        let entry_template_file;
        let feed_template_path = self.xdg_dirs.find_data_file("templates/gemini/atom-feed.xml");
        let feed_template_path = match feed_template_path {
            Some(p) => p,
            _ => {
                eprintln!("Error: Could not find Gemini Atom feed template.");
                exit(1);
            }
        };
        let entry_template_path = self.xdg_dirs.find_data_file("templates/gemini/atom-entry.xml");
        let entry_template_path = match entry_template_path {
            Some(p) => p,
            _ => {
                eprintln!("Error: Could not find Gemini Atom entry template.");
                exit(1);
            }
        };

        feed_template_file = OpenOptions::new()
            .read(true)
            .open(feed_template_path);
        let mut feed_template_file = match feed_template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open Gemini Atom feed template");
                exit(1);
            }
        };
        entry_template_file = OpenOptions::new()
            .read(true)
            .open(entry_template_path);
        let mut entry_template_file = match entry_template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open Gemini Atom entry template");
                exit(1);
            }
        };

        let mut feed_template_buffer = String::new();
        match feed_template_file.read_to_string(&mut feed_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from Gemini Atom feed template");
                exit(1);
            }
        }
        let mut entry_template_buffer = String::new();
        match entry_template_file.read_to_string(&mut entry_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from Gemini Atom entry template");
                exit(1);
            }
        }

        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        match tt.add_template("feed", &feed_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error could not parse gemini feed template file");
                exit(1);
            }
        }
        match tt.add_template("entry", &entry_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error could not parse gemini entry template file");
                exit(1);
            }
        }

        // Generate all entry listings and add to a vector which is used in an AtomFeedContext.
        let mut entries: Vec<String> = Vec::new();
        for post in &self.posts {
            let dt: DateTime<Local> = Local.from_local_datetime(&post.date).unwrap();
            let entry_context = AtomEntryContext {
                site: self.config.site.clone(),
                post: post.clone(),
                rfc_date: dt.to_rfc3339(),
            };
            entries.push(tt.render("entry", &entry_context).unwrap());
        }

        // Generate feed.
        let dt: DateTime<Local> = Local.from_local_datetime(&self.posts[0].date).unwrap();
        let feed_context = AtomFeedContext {
            site: self.config.site.clone(),
            last_updated: dt.to_rfc3339(),
            entries: entries,
        };
        let rendered_feed = tt.render("feed", &feed_context).unwrap();

        println!("Writing gemini Atom feed");

        let feed_path: PathBuf = [
            &self.config.site.gemini_root,
            "index.xml",
        ].iter().collect();

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&feed_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &feed_path.to_string_lossy());
                exit(1);
            }
        };

        match output.write_all(rendered_feed.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: Could not write to {}", &feed_path.to_string_lossy());
                exit(1);
            }
        }
    }

    fn generate_html_atom_feed(&self) {
        let feed_template_file;
        let entry_template_file;
        let feed_template_path = self.xdg_dirs.find_data_file("templates/html/atom-feed.xml");
        let feed_template_path = match feed_template_path {
            Some(p) => p,
            _ => {
                eprintln!("Error: Could not find HTML Atom feed template.");
                exit(1);
            }
        };
        let entry_template_path = self.xdg_dirs.find_data_file("templates/html/atom-entry.xml");
        let entry_template_path = match entry_template_path {
            Some(p) => p,
            _ => {
                eprintln!("Error: Could not find HTML Atom entry template.");
                exit(1);
            }
        };

        feed_template_file = OpenOptions::new()
            .read(true)
            .open(feed_template_path);
        let mut feed_template_file = match feed_template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open HTML Atom feed template");
                exit(1);
            }
        };
        entry_template_file = OpenOptions::new()
            .read(true)
            .open(entry_template_path);
        let mut entry_template_file = match entry_template_file {
            Ok(t) => t,
            Err(_) => {
                eprintln!("Error: Could not open HTML Atom entry template");
                exit(1);
            }
        };

        let mut feed_template_buffer = String::new();
        match feed_template_file.read_to_string(&mut feed_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read HTML Gemini Atom feed template");
                exit(1);
            }
        }
        let mut entry_template_buffer = String::new();
        match entry_template_file.read_to_string(&mut entry_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not read from HTML Atom entry template");
                exit(1);
            }
        }

        let mut tt = TinyTemplate::new();
        tt.set_default_formatter(&tinytemplate::format_unescaped);
        match tt.add_template("feed", &feed_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error could not parse HTML feed template file");
                exit(1);
            }
        }
        match tt.add_template("entry", &entry_template_buffer) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error could not parse HTML entry template file");
                exit(1);
            }
        }

        // Generate all entry listings and add to a vector which is used in an AtomFeedContext.
        let mut entries: Vec<String> = Vec::new();
        for post in &self.posts {
            let dt: DateTime<Local> = Local.from_local_datetime(&post.date).unwrap();
            let entry_context = AtomEntryContext {
                site: self.config.site.clone(),
                post: post.clone(),
                rfc_date: dt.to_rfc3339(),
            };
            entries.push(tt.render("entry", &entry_context).unwrap());
        }

        // Generate feed.
        let dt: DateTime<Local> = Local.from_local_datetime(&self.posts[0].date).unwrap();
        let feed_context = AtomFeedContext {
            site: self.config.site.clone(),
            last_updated: dt.to_rfc3339(),
            entries: entries,
        };
        let rendered_feed = tt.render("feed", &feed_context).unwrap();

        println!("Writing HTML Atom feed");

        let feed_path: PathBuf = [
            &self.config.site.html_root,
            "index.xml",
        ].iter().collect();

        let output = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&feed_path);
        let mut output = match output {
            Ok(o) => o,
            Err(_) => {
                eprintln!("Error: Could not open {} for writing", &feed_path.to_string_lossy());
                exit(1);
            }
        };

        match output.write_all(rendered_feed.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                eprintln!("Error: Could not write to {}", &feed_path.to_string_lossy());
                exit(1);
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

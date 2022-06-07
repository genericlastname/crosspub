pub mod about;
pub mod config;
pub mod contexts;
pub mod crosspub;
pub mod frontmatter;
pub mod gemtext;
pub mod post;
pub mod topic;

use std::fs;
use std::process::exit;
use std::path::PathBuf;

use clap::Parser;
use xdg;

use crosspub::{Args, CrossPub};

fn main() {
    let mut args = Args::parse();

    // Initialize directory structure then quit.
    if args.init {
        let xdg_dir = xdg::BaseDirectories::with_prefix("crosspub").unwrap();
        let config_path: PathBuf = [
            xdg_dir.get_config_home(),
            PathBuf::from("config.toml")
        ].iter().collect();
        match fs::copy(
            "/usr/share/crosspub/config.toml",
            config_path) {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Could not copy default config");
                exit(1);
            }
        }
        match fs::create_dir("./posts") {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Couldn't create posts/ directory");
                exit(1);
            }
        }
        match fs::create_dir("./topics") {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Error: Couldn't create topics/ directory");
                exit(1);
            }
        }
        match fs::create_dir("~/.config/crosspub") {
            _ => {}
        }
        println!("Initialized crosspub directories and created config.\n\n\
            Blogs/articles go in posts/\n\
            Wikis/digital gardens go in topics/");
        exit(0);
    }

    if args.dir.is_none() {
        args.dir = Some(PathBuf::from("."));
    }

    // Load config
    let xdg_dirs = xdg::BaseDirectories::with_prefix("crosspub").unwrap();
    let config_path: PathBuf;

    if !args.config.is_none() {
        config_path = args.config.clone().unwrap();
    } else {
        config_path = match xdg_dirs.find_config_file("config.toml") {
            Some(p) => p,
            None => {
                eprintln!("Error: could not find config file.");
                exit(1);
            }
        };
    }
    let config_contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Error: could not open config file {}.", config_path.to_string_lossy());
            exit(1);
        }
    };
    let config = match toml::from_str(&config_contents) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Error: could not parse config.toml.");
            exit(1);
        }
    };
    
    let crosspub = CrossPub::new(&config, &args);
    crosspub.write();

    println!("Finished");
}

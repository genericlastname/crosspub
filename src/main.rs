pub mod about;
pub mod crosspub;
pub mod config;
pub mod frontmatter;
pub mod gemtext;
pub mod post;
pub mod topic;

use std::process::exit;
use std::path::PathBuf;

use clap::Parser;

use crosspub::{Args, CrossPub};

fn main() {
    let mut args = Args::parse();
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

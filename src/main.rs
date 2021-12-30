#![feature(iter_intersperse)]

use clap::{AppSettings, Parser, Subcommand};

use std::fs;

/// Parsing of CSS
mod css;
/// Parsing of HTML to DOM
mod html;
/// Application of CSS styles to HTML
#[allow(dead_code)]
mod style;
/// Fetching of resources from the web
mod web;


#[derive(Parser)]
#[clap(author, version, about)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
struct App {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a single file
    Parse { filename: String },
    /// Pull a webpage and apply linked styles
    Request { url: String },
}

fn main() {
    let args = App::parse();
    match args.command {
        Commands::Parse { filename } => parse_file(filename.as_str()),
        Commands::Request { url } => request_url(url.as_str()),
    }
}

fn parse_file(filename: &str) {
    let data = fs::read_to_string(filename).expect("Could not read file");
    if filename.ends_with(".css") {
        let result = css::stylesheet(data.as_str());
        return match result {
            Ok((rem, parsed)) => {
                println!("{:#?}", parsed);
                if !rem.is_empty() {
                    eprintln!("Could not parse: {}", rem);
                }
            }
            Err(_) => eprintln!("Could not parse CSS"),
        };
    } else if filename.ends_with(".html") {
        let result = html::document(data.as_str());
        return match result {
            Ok((rem, parsed)) => {
                println!("{:#?}", parsed);
                if !rem.is_empty() {
                    eprintln!("Could not parse: {}", rem);
                }
            }
            Err(_) => eprintln!("Could not parse HTML"),
        };
    } else {
        eprintln!("Could not determine filetype");
    }
}

fn request_url(url: &str) {
    let page = web::Page::browse(url);
    dbg!(page);
}

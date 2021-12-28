#![feature(iter_intersperse)]

use clap::{AppSettings, Parser, Subcommand};

use crate::style::StyledElement;
use std::fs;

/// Parsing of CSS
mod css;
/// Parsing of HTML to DOM
mod html;
/// Application of CSS styles to HTML
#[allow(dead_code)]
mod style;

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
    use html::DOMContent;
    use reqwest::blocking;

    let url = url::Url::parse(url).expect("Could not parse URL");
    let resp = blocking::get(url.clone())
        .expect("Could not request URL")
        .text()
        .expect("Could not get response text");
    let doc = html::document(resp.as_str())
        .expect("Could not parse HTML")
        .1;
    let mut s_tree: StyledElement = doc.clone().into();
    if let Some(head) = doc.get_elements_by_name("head", false).get(0) {
        head.get_elements_by_name("link", false)
            .iter()
            .filter(|l| l.get_attribute("rel") == Some(&"stylesheet".into()))
            .for_each(|s| {
                if let Some(href) = s.get_attribute("href") {
                    if let Ok(link) = url.join(href) {
                        let resp = blocking::get(link)
                            .expect("Could not request URL")
                            .text()
                            .expect("Could not get response text");
                        let sheet = css::stylesheet(resp.as_str())
                            .expect("Could not parse CSS")
                            .1;
                        s_tree.apply_styles(sheet.rules);
                    }
                }
            });
        head.get_elements_by_name("style", false)
            .iter()
            .for_each(|e| {
                if let Some(DOMContent::Text(t)) = e.contents.get(0) {
                    if let Ok((_, sheet)) = css::stylesheet(t.as_str()) {
                        s_tree.apply_styles(sheet.rules);
                    }
                }
            })
    }
    dbg!(s_tree);
}

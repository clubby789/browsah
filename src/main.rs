#![feature(iter_intersperse)]

use clap::{AppSettings, Parser, Subcommand};

use crate::layout::{create_layout, LayoutBox, Rect};
use std::fs;
use crate::display::{paint};

/// Parsing of CSS
mod css;
/// Parsing of HTML to DOM
mod html;
/// Translation of a [`StyledElement`] tree into a tree of boxes
mod layout;
/// Application of CSS styles to HTML
#[allow(dead_code)]
mod style;
/// Fetching of resources from the web
mod web;
mod display;

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
    /// Pull a webpage and apply linked styles, then render to an image
    Request { url: String, output: String },
}

fn main() {
    let args = App::parse();
    match args.command {
        Commands::Parse { filename } => parse_file(filename.as_str()),
        Commands::Request { url, output} => render_from_url(url.as_str(), output),
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

fn render_from_url(url: &str, output: String) {
    let layout = request_url(url);
    let canvas = paint(&layout, Rect {x: 0, y: 0, width: 500, height: 500});
    let img = canvas.render();
    img.save(output).expect("Could not save to file");

}

fn request_url(url: &str) -> LayoutBox {
    let page = web::Page::browse(url);
    let style = page.style_tree;
    create_layout(&style, (500, 500))
}

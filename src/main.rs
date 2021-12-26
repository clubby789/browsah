#![feature(iter_intersperse)]

extern crate clap;
extern crate nom;
use clap::Parser;

use std::fs;
use crate::style::construct_style_tree;

/// Parsing of HTML to DOM
mod html;
/// Parsing of CSS
mod css;
/// Application of CSS styles to HTML
mod style;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, help = "CSS File")]
    pub css: Option<String>,

    #[clap(short, long, help = "HTML File")]
    pub html: Option<String>,
}

fn main() {
    let args = Args::parse();
    let html = if let Some(file) = args.html {
        let data = fs::read_to_string(file).unwrap();
        let (remaining, parsed) = html::document(data.as_str()).expect("Could not parse HTML");
        if remaining.len() > 0 {
            eprintln!("Could not parse: {}", remaining);
        }
        Some(parsed)
    } else {
        None
    };

    let css = if let Some(file) = args.css {
        let data = fs::read_to_string(file).unwrap();
        let (remaining, parsed) = css::stylesheet(data.as_str()).expect("Could not parse CSS");
        if remaining.len() > 0 {
            eprintln!("Could not parse: {}", remaining);
        }
        Some(parsed)
    } else {None};
    if let Some(dom) = html {
        if let Some(ss) = css {
            let tree = construct_style_tree(dom, ss);
            dbg!(tree);
        } else {
            dbg!(dom);
        }
    } else {
        if let Some(ss) = css {
            dbg!(ss);
        }
    }
}

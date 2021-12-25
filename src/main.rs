extern crate clap;
extern crate nom;
use clap::Parser;

use std::fs;

mod css;
mod html;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, help = "CSS File")]
    pub css: Option<String>,

    #[clap(short, long, help = "HTML File")]
    pub html: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(file) = args.html {
        let data = fs::read_to_string(file).unwrap();
        let (remaining, parsed) = html::parse_dom_node(data.as_str()).expect("Could not parse HTML");
        println!("Parsed: {:#?}", parsed);
        if remaining.len() > 0 {
            eprintln!("Could not parse: {}", remaining);
        }
    }
    if let Some(file) = args.css {
        let data = fs::read_to_string(file).unwrap();
        let (remaining, parsed) = css::stylesheet(data.as_str()).expect("Could not parse CSS");
        println!("Parsed: {:#?}", parsed);
        if remaining.len() > 0 {
            eprintln!("Could not parse: {}", remaining);
        }
    }
}

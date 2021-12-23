extern crate nom;
extern crate clap;
extern crate reqwest;

use clap::Parser;

mod css;
mod html;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    pub css: Option<String>,

    #[clap(short, long)]
    pub html: Option<String>,
}

fn main() {
    let args = Args::parse();
    if let Some(url) = args.css {
        dbg!(css::parse_stylesheet(reqwest::blocking::get(url).unwrap().text().unwrap().as_str()).unwrap());
    }
}

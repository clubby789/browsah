extern crate clap;
extern crate nom;
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
    println!("Hello, world");
}

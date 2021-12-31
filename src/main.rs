#![feature(iter_intersperse)]
#![feature(int_abs_diff)]

use crate::display::paint;
use crate::layout::{create_layout, LayoutBox, Rect};
use tracing::{info, span, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Parsing of CSS
mod css;
mod display;
/// Parsing of HTML to DOM
mod html;
/// Translation of a [`StyledElement`] tree into a tree of boxes
mod layout;
/// Application of CSS styles to HTML
#[allow(dead_code)]
mod style;
/// Fetching of resources from the web
mod web;

struct Args {
    pub input: String,
    pub output: String,
    pub trace: bool,
}

fn main() {
    let args = parse_args().expect("Could not parse arguments");
    if args.trace {
        tracing_subscriber::fmt::fmt()
            .with_span_events(FmtSpan::ACTIVE)
            .with_max_level(Level::DEBUG)
            .with_env_filter(EnvFilter::from_default_env())
            .finish()
            .init();
        info!("Logger initialized");
    }

    render_from_url(args.input.as_str(), args.output);
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    let args = Args {
        input: pargs.free_from_str()?,
        output: pargs.free_from_str()?,
        trace: pargs.contains(["--trace", "-t"]),
    };
    Ok(args)
}

fn render_from_url(url: &str, output: String) {
    let layout = request_url(url);
    let canvas = paint(
        &layout,
        Rect {
            x: 0,
            y: 0,
            width: 1600,
            height: 1080,
        },
    );
    let img = canvas.render();
    let span = span!(Level::DEBUG, "Saving result");
    let _enter = span.enter();
    img.save(output).expect("Could not save to file");
}

fn request_url(url: &str) -> LayoutBox {
    let page = web::Page::browse(url);
    let style = page.style_tree;
    create_layout(&style, (1600, 1080))
}

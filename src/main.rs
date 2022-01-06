#![feature(int_abs_diff)]

use crate::layout::{create_layout, LayoutBox, Rect};
use paint::paint;
use tracing::{info, span, Level};

/// Fetching of resources from the web
mod web;
/// Application of CSS styles to HTML
mod style;
/// Translation of a [`style::StyledElement`] tree into a tree of boxes
mod layout;
/// Conversion into list of [`display::DisplayCommand`]
mod display;
/// Painting [`display::DisplayCommand`]s onto a [`paint::Canvas`]
mod paint;



struct Args {
    pub input: String,
    pub output: String,
    pub trace: bool,
    pub v1: bool,
    pub v2: bool,
}

fn main() {
    use tracing_subscriber::{filter::FilterFn, fmt::format::FmtSpan, prelude::*};
    let args = parse_args().expect("Could not parse arguments");
    if args.trace {
        let level = {
            if args.v2 {
                Level::TRACE
            } else if args.v1 {
                Level::DEBUG
            } else {
                Level::INFO
            }
        };
        let to_log = ["browsah", "html", "css"];
        let fmt_layer = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::ACTIVE);
        let filter = FilterFn::new(move |meta| {
            to_log.iter().any(|cr| meta.target().starts_with(cr)) && meta.level() <= &level
        });
        tracing_subscriber::registry()
            .with(fmt_layer.with_filter(filter))
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
        trace: pargs.contains(["-t", "--trace"]),
        v1: pargs.contains("-v"),
        v2: pargs.contains("-vv"),
    };
    Ok(args)
}

fn render_from_url(url: &str, output: String) {
    let layout = request_url(url);
    let canvas = paint(
        &layout,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 1600.0,
            height: 1080.0,
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

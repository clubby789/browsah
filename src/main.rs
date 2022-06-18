use crate::layout::{create_layout, LayoutBox, Rect};
use paint::paint;
use tracing::{info, span, Level};

/// Conversion into list of [`display::DisplayCommand`]
mod display;
/// Translation of a [`style::StyledElement`] tree into a tree of boxes
mod layout;
/// Painting [`display::DisplayCommand`]s onto a [`paint::Canvas`]
mod paint;
/// Application of CSS styles to HTML
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
        tracing_subscriber::fmt::init();
        info!("Initialized");
    }
    render_from_url(args.input.as_str(), args.output);
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();
    let args = Args {
        input: pargs.free_from_str()?,
        output: pargs.free_from_str()?,
        trace: pargs.contains(["-t", "--trace"]),
    };
    Ok(args)
}

fn render_from_url(url: &str, output: String) {
    let page = web::Page::browse(url);
    let stylesheets = page.get_stylesheet_text();
    let styles = page.get_styles(&stylesheets);
    info!("Applying stylesheets");
    for sheet in styles {
        page.style_tree.borrow_mut().apply_styles(&sheet.rules);
    }
    let style = page.style_tree.borrow();
    info!("Performing layout");
    let layout = create_layout(&style, (1600, 1080));
    info!("Painting canvas");
    let canvas = paint(
        &layout,
        Rect {
            x: 0.0,
            y: 0.0,
            width: 1600.0,
            height: 1080.0,
        },
    );
    info!("Rendering to image");
    let img = canvas.render();
    let span = span!(Level::DEBUG, "Saving result");
    let _enter = span.enter();
    img.save(output).expect("Could not save to file");
}

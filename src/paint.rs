use crate::display::DisplayCommand;
use crate::{display, LayoutBox, Rect};
use css::ColorValue;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use image::{GenericImage, ImageBuffer, Rgba};
use once_cell::sync::Lazy;
use std::ops::Deref;
use tracing::{span, Level};

static ARIAL_TTF: &[u8] = include_bytes!("../resources/arial.ttf");
static ARIAL: Lazy<Font> =
    Lazy::new(|| Font::from_bytes(ARIAL_TTF, fontdue::FontSettings::default()).unwrap());

pub struct Canvas {
    pub pixels: ImageBuffer<Rgba<u8>, Vec<u8>>,
    width: u32,
    height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: ImageBuffer::new(width, height),
            width,
            height,
        }
    }
    fn paint_command(&mut self, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::SolidBlock(color, rect) => {
                let x0 = rect.x.clamp(0.0, self.width as f64) as u32;
                let y0 = rect.y.clamp(0.0, self.height as f64) as u32;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f64) as u32;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f64) as u32;
                for y in y0..y1 {
                    for x in x0..x1 {
                        // SAFETY: Start and end X are clamped to self.width and self.height, and
                        // those values are used to initialise the buffer
                        unsafe {
                            self.pixels.unsafe_put_pixel(x, y, color_to_pix(color));
                        }
                    }
                }
            }
            DisplayCommand::Text(text, size, rect, color) => {
                let x0 = rect.x.clamp(0.0, self.width as f64) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f64) as usize;

                let settings = LayoutSettings {
                    x: x0 as f32,
                    y: y0 as f32,
                    max_width: Some(rect.width as f32),
                    ..Default::default()
                };

                let mut layout = get_rasterized_layout(text, *size as f32, &settings);
                for glyph in layout.glyphs() {
                    let y_start = glyph.y as u32;
                    let x_start = glyph.x as u32;
                    let (_, bitmap) = ARIAL.rasterize(glyph.parent, *size as f32);
                    for (yb, y) in (y_start..y_start + glyph.height as u32).enumerate() {
                        for (xb, x) in (x_start..x_start + glyph.width as u32).enumerate() {
                            let percent = (bitmap[xb + yb * glyph.width] as f32) / 255.0;
                            let pixel = self.pixels.get_pixel_mut(x, y);
                            *pixel = interpolate_rgba(*pixel, color_to_pix(color), percent);
                        }
                    }
                }
            }
        }
    }
    /// Consumes the canvas, returning the rendered ImageBuffer
    pub fn render(self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        self.pixels
    }
}

pub fn paint(root: &LayoutBox, bounds: Rect) -> Canvas {
    let span = span!(Level::DEBUG, "Painting page");
    let _enter = span.enter();
    let span2 = span!(Level::DEBUG, "Generating DisplayList");
    let _enter2 = span2.enter();
    let cmds = display::build_display_list(root);
    let mut canvas = Canvas::new(bounds.width as u32, bounds.height as u32);
    cmds.into_iter().for_each(|cmd| canvas.paint_command(&cmd));
    canvas
}

pub fn get_rasterized_layout(text: &str, font_size: f32, settings: &LayoutSettings) -> Layout {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(settings);
    layout.append(&[ARIAL.deref()], &TextStyle::new(text, font_size, 0));
    layout
}

fn color_to_pix(val: &ColorValue) -> image::Rgba<u8> {
    image::Rgba::from([val.r, val.g, val.b, val.a])
}

fn interpolate_rgba(from: Rgba<u8>, to: Rgba<u8>, fac: f32) -> Rgba<u8> {
    if from == to {
        return from;
    }
    let r = interpolate_val(from[0], to[0], fac);
    let g = interpolate_val(from[1], to[1], fac);
    let b = interpolate_val(from[2], to[2], fac);
    let a = interpolate_val(from[3], to[3], fac);
    Rgba([r, g, b, a])
}

fn interpolate_val(from: u8, to: u8, fac: f32) -> u8 {
    use std::cmp::Ordering;
    match from.cmp(&to) {
        Ordering::Equal => from,
        Ordering::Less => to - ((to - from) as f32 * fac) as u8,
        Ordering::Greater => from - ((from - to) as f32 * fac) as u8,
    }
}

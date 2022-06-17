use crate::display::DisplayCommand;
use crate::{display, LayoutBox, Rect};
use css::{ColorValue, WHITE};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use image::{ImageBuffer, Rgba};
use once_cell::sync::Lazy;
use tracing::{span, Level};

static ARIAL_TTF: &[u8] = include_bytes!("../resources/arial.ttf");
static ARIAL: Lazy<Font> =
    Lazy::new(|| Font::from_bytes(ARIAL_TTF, fontdue::FontSettings::default()).unwrap());

pub struct Canvas {
    pub pixels: Vec<ColorValue>,
    width: usize,
    height: usize,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: std::iter::repeat(WHITE).take(width * height).collect(),
            width,
            height,
        }
    }
    fn paint_command(&mut self, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::SolidBlock(color, rect) => {
                let x0 = rect.x.clamp(0.0, self.width as f64) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f64) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f64) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f64) as usize;
                for y in y0..y1 {
                    for x in x0..x1 {
                        self.pixels[x + y * self.width] = *color;
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
                    let y_start = glyph.y as usize;
                    let x_start = glyph.x as usize;
                    let (_, bitmap) = ARIAL.rasterize(glyph.parent, *size as f32);
                    for (yb, y) in (y_start..y_start + glyph.height).enumerate() {
                        for (xb, x) in (x_start..x_start + glyph.width).enumerate() {
                            let percent = (bitmap[xb + yb * glyph.width] as f32) / 255.0;
                            let orig = self.pixels[x + y * self.width];
                            self.pixels[x + y * self.width] =
                                css::interpolate_color(orig, *color, percent);
                        }
                    }
                }
            }
        }
    }
    pub fn render(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let span = span!(Level::DEBUG, "Rendering to pixel buffer");
        let _enter = span.enter();
        let (w, h) = (self.width, self.height);
        let buffer: Vec<Rgba<u8>> = self.pixels.iter().map(color_to_pix).collect();
        assert_eq!(w * h, buffer.len());
        // SAFETY: We've asserted that w*h won't go out of bounds
        image::ImageBuffer::from_fn(w as u32, h as u32, |x, y| unsafe {
            *buffer.get_unchecked(y as usize * w + x as usize)
        })
    }
}

pub fn paint(root: &LayoutBox, bounds: Rect) -> Canvas {
    let span = span!(Level::DEBUG, "Painting page");
    let _enter = span.enter();
    let span2 = span!(Level::DEBUG, "Generating DisplayList");
    let _enter2 = span2.enter();
    let cmds = display::build_display_list(root);
    let mut canvas = Canvas::new(bounds.width as usize, bounds.height as usize);
    cmds.into_iter().for_each(|cmd| canvas.paint_command(&cmd));
    canvas
}

pub fn get_rasterized_layout(text: &str, font_size: f32, settings: &LayoutSettings) -> Layout {
    let fonts = &[ARIAL.clone()];
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(settings);
    layout.append(fonts, &TextStyle::new(text, font_size, 0));
    layout
}

fn color_to_pix(val: &ColorValue) -> image::Rgba<u8> {
    image::Rgba::from([val.r, val.g, val.b, val.a])
}

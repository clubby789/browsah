use crate::layout::{BoxContentType, LayoutBox, Rect};
use crate::style::StyleMap;
use css::{ColorValue, Value, BLACK, WHITE};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use image::{ImageBuffer, Rgba};
use lazy_static::lazy_static;
use tracing::{span, Level};

static ARIAL_TTF: &[u8] = include_bytes!("../resources/arial.ttf");
lazy_static! {
    static ref ARIAL: Font = Font::from_bytes(ARIAL_TTF, fontdue::FontSettings::default()).unwrap();
}

#[derive(Debug)]
pub enum DisplayCommand {
    SolidBlock(ColorValue, Rect),
    Text(String, f64, Rect, ColorValue),
}

pub fn build_display_list(root: &LayoutBox) -> Vec<DisplayCommand> {
    let mut list = vec![render_background(root)];
    list.extend(render_borders(root).unwrap_or_default());
    if let BoxContentType::Text(_) = &root.box_content_type {
        list.extend(render_text(root));
    }
    root.contents.iter().for_each(|c| {
        list.extend(build_display_list(c));
    });
    list
}

fn get_color_value(style: &StyleMap, attr: impl Into<String>) -> Option<&ColorValue> {
    style.get(attr).and_then(|val| {
        if let Value::Color(cv) = val {
            Some(cv)
        } else {
            None
        }
    })
}

fn render_background(root: &LayoutBox) -> DisplayCommand {
    let bg = get_color_value(&root.style, "background")
        .unwrap_or_else(|| get_color_value(&root.style, "background-color").unwrap_or(&WHITE));
    DisplayCommand::SolidBlock(*bg, root.dimensions.border_box())
}

fn render_borders(root: &LayoutBox) -> Option<Vec<DisplayCommand>> {
    let color = get_color_value(&root.style, "border-color")?;
    let mut cmds = Vec::with_capacity(4);
    let dim = root.dimensions;
    let border = dim.border_box();
    // Left
    cmds.push(DisplayCommand::SolidBlock(
        *color,
        Rect {
            x: border.x,
            y: border.y,
            width: dim.border.left as f64,
            height: border.height,
        },
    ));
    // Right
    cmds.push(DisplayCommand::SolidBlock(
        *color,
        Rect {
            x: border.x + border.width - dim.border.right,
            y: border.y,
            width: dim.border.right,
            height: border.height,
        },
    ));
    // Top
    cmds.push(DisplayCommand::SolidBlock(
        *color,
        Rect {
            x: border.x,
            y: border.y,
            width: border.width,
            height: dim.border.top,
        },
    ));
    // Bottom
    cmds.push(DisplayCommand::SolidBlock(
        *color,
        Rect {
            x: border.x,
            y: border.y + border.height - dim.border.bottom,
            width: border.width,
            height: dim.border.bottom,
        },
    ));

    Some(cmds)
}

fn render_text(s: &LayoutBox) -> Vec<DisplayCommand> {
    if let BoxContentType::Text(t) = &s.box_content_type {
        let size = s.font_size;
        vec![DisplayCommand::Text(
            t.clone(),
            size,
            s.dimensions.border_box(),
            BLACK,
        )]
    } else {
        unreachable!()
    }
}

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
        let (w, h) = (self.width as u32, self.height as u32);
        let buffer: Vec<image::Rgba<u8>> = self.pixels.iter().map(color_to_pix).collect();
        image::ImageBuffer::from_fn(w, h, |x, y| buffer[(y * w + x) as usize])
    }
}

fn color_to_pix(val: &ColorValue) -> image::Rgba<u8> {
    image::Rgba::from([val.r, val.g, val.b, val.a])
}

pub fn paint(root: &LayoutBox, bounds: Rect) -> Canvas {
    let span = span!(Level::DEBUG, "Painting page");
    let _enter = span.enter();
    let span2 = span!(Level::DEBUG, "Generating DisplayList");
    let _enter2 = span2.enter();
    let cmds = build_display_list(root);
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

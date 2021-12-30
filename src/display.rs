use image::{ImageBuffer, Rgba};
use crate::css::{ColorValue, Value, WHITE};
use crate::layout::{LayoutBox, Rect};
use crate::style::{StyleMap};

#[derive(Debug)]
pub enum DisplayCommand {
    SolidBlock(ColorValue, Rect)
}

pub fn build_display_list(root: &LayoutBox) -> Vec<DisplayCommand> {
    let mut list = vec![];
    list.push(render_background(root));
    list.extend(render_borders(root).unwrap_or(vec![]));
    root.contents.iter().for_each(|c|{list.extend(build_display_list(c));});
    list
}

fn get_color_value(style: &StyleMap, attr: impl Into<String>) -> Option<&ColorValue> {
    style.get(attr).map(|val| if let Value::Color(cv) = val {Some(cv)} else {None}).flatten()
}

fn render_background(root: &LayoutBox) -> DisplayCommand {
    let bg = get_color_value(&root.style, "background").unwrap_or(&WHITE);
    DisplayCommand::SolidBlock(*bg, root.dimensions.border_box())
}

fn render_borders(root: &LayoutBox) -> Option<Vec<DisplayCommand>> {
    let color = get_color_value(&root.style, "border-color")?;
    let mut cmds = Vec::with_capacity(4);
    let dim = root.dimensions;
    let border = dim.border_box();
    // Left
    cmds.push(DisplayCommand::SolidBlock(color.clone(), Rect {
        x: border.x,
        y: border.y,
        width: dim.border.left,
        height: border.height
    }));
    // Right
    cmds.push(DisplayCommand::SolidBlock(color.clone(), Rect {
        x: border.x + border.width - dim.border.right,
        y: border.y,
        width: dim.border.right,
        height: border.height
    }));
    // Top
    cmds.push(DisplayCommand::SolidBlock(color.clone(), Rect {
        x: border.x,
        y: border.y,
        width: border.width,
        height: dim.border.top
    }));
    // Bottom
    cmds.push(DisplayCommand::SolidBlock(color.clone(), Rect {
        x: border.x,
        y: border.y + border.height - dim.border.bottom,
        width: border.width,
        height: dim.border.bottom
    }));

    Some(cmds)

}

pub struct Canvas {
    pub pixels: Vec<ColorValue>,
    width: usize,
    height: usize
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            pixels: std::iter::repeat(WHITE).take(width * height).collect(),
            width, height
        }
    }
    fn paint_command(&mut self, cmd: &DisplayCommand) {
        match cmd {
            DisplayCommand::SolidBlock(color, rect) => {
                let x0 = rect.x.clamp(0, self.width);
                let y0 = rect.y.clamp(0, self.height);
                let x1 = (rect.x + rect.width).clamp(0, self.width);
                let y1 = (rect.y + rect.height).clamp(0, self.height);
                for y in y0 .. y1 {
                    for x in x0 .. x1 {
                        self.pixels[x + y * self.width] = *color;
                    }
                }
            }
        }
    }
    pub fn render(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let (w, h) = (self.width as u32, self.height as u32);
        let buffer: Vec<image::Rgba<u8>> = self.pixels.iter().map(color_to_pix).collect();
        image::ImageBuffer::from_fn(w, h, |x, y| buffer[(y*w+x) as usize])
    }
}

fn color_to_pix(val: &ColorValue) -> image::Rgba<u8> {
    image::Rgba::from([val.r, val.g, val.b, val.a])
}

pub fn paint(root: &LayoutBox, bounds: Rect) -> Canvas {
    let cmds = build_display_list(root);
    let mut canvas = Canvas::new(bounds.width, bounds.height);
    cmds.into_iter().for_each(|cmd| canvas.paint_command(&cmd));
    canvas
}
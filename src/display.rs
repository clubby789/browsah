use crate::layout::{BoxContentType, LayoutBox, Rect};
use crate::style::StyleMap;
use css::{ColorValue, Value, BLACK, WHITE};

pub enum DisplayCommand<'a> {
    SolidBlock(ColorValue, Rect),
    Text(&'a str, f64, Rect, ColorValue),
}

/// Construct list of [`DisplayCommand`]s from a number of drawable [`LayoutBox`]es
pub fn build_display_list<'a>(root: &'a LayoutBox) -> Vec<DisplayCommand<'a>> {
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

fn get_color_value<'a>(style: &'a StyleMap, attr: &str) -> Option<&'a ColorValue> {
    style.get(attr).and_then(|val| {
        if let Value::Color(cv) = val {
            Some(cv)
        } else {
            None
        }
    })
}

fn render_background<'a>(root: &'a LayoutBox) -> DisplayCommand<'a> {
    let bg = get_color_value(&root.style, "background")
        .unwrap_or_else(|| get_color_value(&root.style, "background-color").unwrap_or(&WHITE));
    DisplayCommand::SolidBlock(*bg, root.dimensions.border_box())
}

fn render_borders<'a>(root: &'a LayoutBox) -> Option<Vec<DisplayCommand<'a>>> {
    let mut cmds = Vec::with_capacity(4);
    let dim = root.dimensions;
    let border = dim.border_box();
    let border_details = root.border.as_ref()?;
    // Left
    cmds.push(DisplayCommand::SolidBlock(
        border_details.left.color.try_to_color().unwrap(),
        Rect {
            x: border.x,
            y: border.y,
            width: dim.border.left as f64,
            height: border.height,
        },
    ));
    // Right
    cmds.push(DisplayCommand::SolidBlock(
        border_details.right.color.try_to_color().unwrap(),
        Rect {
            x: border.x + border.width - dim.border.right,
            y: border.y,
            width: dim.border.right,
            height: border.height,
        },
    ));
    // Top
    cmds.push(DisplayCommand::SolidBlock(
        border_details.top.color.try_to_color().unwrap(),
        Rect {
            x: border.x,
            y: border.y,
            width: border.width,
            height: dim.border.top,
        },
    ));
    // Bottom
    cmds.push(DisplayCommand::SolidBlock(
        border_details.bottom.color.try_to_color().unwrap(),
        Rect {
            x: border.x,
            y: border.y + border.height - dim.border.bottom,
            width: border.width,
            height: dim.border.bottom,
        },
    ));

    Some(cmds)
}

fn render_text<'a>(s: &'a LayoutBox) -> Vec<DisplayCommand<'a>> {
    if let BoxContentType::Text(t) = &s.box_content_type {
        let size = s.font_size;
        vec![DisplayCommand::Text(
            t,
            size,
            s.dimensions.border_box(),
            BLACK,
        )]
    } else {
        unreachable!()
    }
}

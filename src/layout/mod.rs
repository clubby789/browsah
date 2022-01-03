#![allow(dead_code, unused_variables)]

mod properties;

use crate::display::get_rasterized_layout;
use crate::layout::properties::{get_border, get_margins, get_padding, Border, Margin, Padding};
use crate::layout::BoxContentType::Text;
use crate::style::{StyleMap, StyledContent, StyledElement};
use css::{Unit, Value};
use fontdue::layout::LayoutSettings;
use std::str::FromStr;
use tracing::{span, Level};

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    box_type: BoxType,
    pub contents: Vec<LayoutBox>,
    pub style: StyleMap,
    pub box_content_type: BoxContentType,
    pub font_size: f64,
    pub border: Option<Border>,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum BoxContentType {
    Normal,
    Image,
    Text(String),
}

fn get_content_type(content: &StyledContent) -> BoxContentType {
    match content {
        StyledContent::Element(elt) => {
            if elt.name.as_str() == "img" {
                return BoxContentType::Image;
            };
            BoxContentType::Normal
        }
        StyledContent::Text(txt) => BoxContentType::Text(txt.contents.clone()),
    }
}

#[derive(Copy, Clone, Default)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Dimensions {
    pub content: Rect,
    pub margin: EdgeSizes,
    pub border: EdgeSizes,
    pub padding: EdgeSizes,
}

impl Dimensions {
    pub fn padding_box(self) -> Rect {
        self.content.expanded_by(self.padding)
    }
    pub fn border_box(self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }
    pub fn margin_box(self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
enum BoxType {
    Block,
    Inline,
    Anonymous,
}

impl From<&Value> for BoxType {
    fn from(v: &Value) -> Self {
        if let Value::Keyword(k) = v {
            k.as_str().parse().unwrap_or(BoxType::Block)
        } else {
            BoxType::Block
        }
    }
}

impl Default for BoxType {
    fn default() -> Self {
        Self::Block
    }
}

impl FromStr for BoxType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "block" => Self::Block,
            "inline" => Self::Inline,
            _ => Self::default(),
        })
    }
}

#[derive(Copy, Clone, Default)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    fn expanded_by(self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct EdgeSizes {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

pub fn create_layout(root: &StyledElement, viewport_size: (usize, usize)) -> LayoutBox {
    let span = span!(Level::DEBUG, "Creating layout tree");
    let _enter = span.enter();

    let (width, _) = viewport_size;
    let container = Dimensions {
        content: Rect {
            x: 0.0,
            y: 0.0,
            width: width as f64,
            height: 0.0,
        },
        margin: EdgeSizes::default(),
        border: EdgeSizes::default(),
        padding: EdgeSizes::default(),
    };
    let mut root_box = build_layout_tree(root);
    root_box.layout(container);
    root_box
}

fn build_layout_tree(root: &StyledElement) -> LayoutBox {
    let font_size = if let Some(Value::Number(n)) = root.styles.get("font-size") {
        *n
    } else {
        16.0
    };
    let mut root_box = LayoutBox {
        dimensions: Default::default(),
        box_type: root
            .styles
            .get("display")
            .map(|d| d.into())
            .unwrap_or(BoxType::Block),
        contents: vec![],
        style: root.styles.clone(),
        box_content_type: BoxContentType::Normal,
        font_size,
        border: None,
    };
    for child in &root.contents {
        match child {
            StyledContent::Element(elt) => {
                match elt
                    .styles
                    .get("display")
                    .map(|d| d.into())
                    .unwrap_or(BoxType::Block)
                {
                    BoxType::Block => root_box.contents.push(build_layout_tree(elt)),
                    BoxType::Inline => root_box
                        .get_inline_container()
                        .contents
                        .push(build_layout_tree(elt)),
                    _ => {}
                }
            }
            StyledContent::Text(text) => {
                let box_type = text
                    .styles
                    .get("display")
                    .map(|d| d.into())
                    .unwrap_or(BoxType::Block);
                let the_box = LayoutBox {
                    dimensions: Default::default(),
                    box_type,
                    contents: vec![],
                    style: root.styles.clone(),
                    box_content_type: Text(text.contents.clone()),
                    font_size,
                    border: None,
                };
                match box_type {
                    BoxType::Block => root_box.contents.push(the_box),
                    BoxType::Inline => root_box.get_inline_container().contents.push(the_box),
                    _ => {}
                }
            }
        }
    }
    root_box
}

fn calculate_font_size(s: &StyleMap, parent_size: f64) -> f64 {
    match s.get("font-size") {
        Some(Value::Keyword(_)) => None,
        Some(Value::Number(n)) => Some(*n),
        Some(Value::Percentage(n)) => Some(*n * parent_size),
        Some(Value::Length(n, unit)) => match unit {
            Unit::Px => Some(*n),
            Unit::Em => Some(*n * parent_size),
            _ => None,
        },
        _ => None,
    }
    .unwrap_or(parent_size)
}

impl LayoutBox {
    fn new(box_type: BoxType, font_size: f64) -> LayoutBox {
        LayoutBox {
            box_type,
            contents: vec![],
            dimensions: Default::default(),
            style: Default::default(),
            box_content_type: BoxContentType::Normal,
            font_size,
            border: None,
        }
    }
    fn layout(&mut self, container: Dimensions) {
        match self.box_type {
            BoxType::Block => self.layout_block(container),
            _ => todo!(),
        }
        for child in self.contents.iter_mut() {
            child.font_size = calculate_font_size(&child.style, self.font_size);
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox {
        match self.box_type {
            BoxType::Inline | BoxType::Anonymous => self,
            BoxType::Block => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                match self.contents.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::Anonymous,
                        ..
                    }) => {}
                    _ => self
                        .contents
                        .push(LayoutBox::new(BoxType::Anonymous, self.font_size)),
                }
                self.contents.last_mut().unwrap()
            }
        }
    }

    fn layout_block(&mut self, container: Dimensions) {
        self.calculate_block_width(container);
        self.calculate_block_position(container);
        self.layout_block_children();
        self.calculate_block_height();
    }

    fn calculate_block_width(&mut self, container: Dimensions) {
        if let BoxContentType::Text(_) = self.box_content_type {
            return self.calculate_text_block_width(container);
        }
        let style = &self.style;
        let auto = Value::Keyword("auto".to_string());
        let default = Value::Number(0.0);
        let mut width = &style.get("width").cloned().unwrap_or_else(|| auto.clone());
        let margins = get_margins(style);
        let Margin {
            left: mut margin_left,
            right: mut margin_right,
            ..
        } = margins;
        let border = get_border(style);
        let (border_left, border_right) = (border.left.width.clone(), border.right.width.clone());
        self.border = Some(border);
        let Padding {
            left: padding_left,
            right: padding_right,
            ..
        } = get_padding(style);
        let total_width = [
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            width,
        ]
        .iter()
        .map(|v| v.try_to_px(self.font_size).unwrap_or(0.0))
        .sum::<f64>();
        if width != &auto && total_width > container.content.width {
            if margin_left == auto {
                margin_left = default.clone();
            }
            if margin_right == auto {
                margin_right = default.clone();
            }
        }
        let underflow = container.content.width as isize - total_width as isize;
        // These values must be created outside the match so they live long enough
        let underflow_val = Value::Number(underflow as f64);
        let adjusted_margin_right = Value::Number(
            (margin_right.try_to_px(self.font_size).unwrap_or(0.0) as isize + underflow) as f64,
        );
        let half_underflow = Value::Number(underflow as f64 / 2.0);
        match (width == &auto, margin_left == auto, margin_right == auto) {
            (false, false, false) => margin_right = adjusted_margin_right,
            (false, false, true) => margin_right = underflow_val,
            (false, true, false) => margin_left = underflow_val,
            (true, _, _) => {
                if underflow >= 0 {
                    width = &underflow_val
                } else {
                    width = &default;
                    margin_right = adjusted_margin_right;
                }
            }
            (false, true, true) => {
                margin_left = half_underflow.clone();
                margin_right = half_underflow;
            }
        }
        let dim = &mut self.dimensions;
        dim.content.width = width.try_to_px(self.font_size).unwrap();
        dim.padding.left = padding_left.try_to_px(self.font_size).unwrap();
        dim.padding.right = padding_right.try_to_px(self.font_size).unwrap();
        dim.border.left = border_left.try_to_px(self.font_size).unwrap();
        dim.border.right = border_right.try_to_px(self.font_size).unwrap();
        dim.margin.left = margin_left.try_to_px(self.font_size).unwrap();
        dim.margin.right = margin_right.try_to_px(self.font_size).unwrap();
    }

    fn calculate_text_block_width(&mut self, container: Dimensions) {
        let settings = LayoutSettings {
            max_width: Some(container.border_box().width as f32),
            ..Default::default()
        };
        if let BoxContentType::Text(s) = &self.box_content_type {
            let font_size = self
                .style
                .get("font-size")
                .and_then(|v| v.try_to_px(self.font_size))
                .unwrap_or(self.font_size);
            let mut layout = get_rasterized_layout(s, font_size as f32, &settings);
            let dim = &mut self.dimensions;
            dim.padding.left = 0.0;
            dim.padding.right = 0.0;
            dim.border.left = 0.0;
            dim.border.right = 0.0;
            dim.margin.left = 0.0;
            dim.content.width = layout
                .glyphs()
                .iter()
                .max_by_key(|v| v.x as usize)
                .map(|x| x.x + x.width as f32)
                .unwrap_or(0.0) as f64
                + 2.0;
            dim.content.height = layout.height() as f64;
        } else {
            unreachable!();
        }
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = &self.style;
        let dim = &mut self.dimensions;
        let zero = Value::Number(0.0);
        let Margin {
            top: margin_top,
            bottom: margin_bottom,
            ..
        } = get_margins(style);
        dim.margin.top = margin_top.try_to_px(self.font_size).unwrap();
        dim.margin.bottom = margin_bottom.try_to_px(self.font_size).unwrap();

        let Border {
            top: border_top,
            bottom: border_bottom,
            ..
        } = get_border(style);
        dim.border.top = border_top.width.try_to_px(self.font_size).unwrap();
        dim.border.bottom = border_bottom.width.try_to_px(self.font_size).unwrap();

        let Padding {
            top: padding_top,
            bottom: padding_bottom,
            ..
        } = get_padding(style);
        dim.padding.top = padding_top.try_to_px(self.font_size).unwrap();
        dim.padding.bottom = padding_bottom.try_to_px(self.font_size).unwrap();
        dim.content.x =
            containing_block.content.x + dim.margin.left + dim.border.left + dim.padding.left;
        dim.content.y = containing_block.content.height
            + containing_block.content.y
            + dim.margin.top
            + dim.border.top
            + dim.padding.top;
    }

    fn layout_block_children(&mut self) {
        let dim = &mut self.dimensions;
        self.contents.iter_mut().for_each(|c| {
            c.layout(*dim);
            dim.content.height += c.dimensions.margin_box().height
        });
    }

    fn calculate_block_height(&mut self) {
        if let Some(n) = self
            .style
            .get("height")
            .map(|v| v.try_to_px(self.font_size).unwrap_or(0.0))
        {
            self.dimensions.content.height = n;
        }
    }
}

#![allow(dead_code, unused_variables)]

use crate::css::{NumericValue, TextValue, Value};
use crate::style::{StyleMap, StyledContent, StyledElement};
use std::str::FromStr;
use crate::css::TextValue::Keyword;

#[derive(Debug)]
pub struct LayoutBox {
    pub dimensions: Dimensions,
    box_type: BoxType,
    pub contents: Vec<LayoutBox>,
    pub style: StyleMap,
}

#[derive(Copy, Clone, Debug, Default)]
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

#[derive(Copy, Clone, Debug)]
enum BoxType {
    Block,
    Inline,
    Anonymous,
}

impl From<&Value> for BoxType {
    fn from(v: &Value) -> Self {
        if let Value::Textual(TextValue::Keyword(k)) = v {
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

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
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
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
}

pub fn create_layout(root: &StyledElement, viewport_size: (usize, usize)) -> LayoutBox {
    let (width, _) = viewport_size;
    let container = Dimensions {
        content: Rect {
            x: 0,
            y: 0,
            width,
            height: 0,
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
    let mut root_box = LayoutBox {
        dimensions: Default::default(),
        box_type: root
            .styles
            .get("display")
            .map(|d| d.into())
            .unwrap_or(BoxType::Block),
        contents: vec![],
        style: root.styles.clone(),
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
            },
            StyledContent::Text(text) => {
                todo!()
            }
        }
    }
    root_box
}

impl LayoutBox {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            contents: vec![],
            dimensions: Default::default(),
            style: Default::default()
        }
    }
    fn layout(&mut self, container: Dimensions) {
        match self.box_type {
            BoxType::Block => self.layout_block(container),
            _ => todo!(),
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox {
        match self.box_type {
            BoxType::Inline | BoxType::Anonymous => self,
            BoxType::Block => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                match self.contents.last() {
                    Some(&LayoutBox { box_type: BoxType::Anonymous,..}) => {}
                    _ => self.contents.push(LayoutBox::new(BoxType::Anonymous))
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
        let style = &self.style;
        let auto = Value::Textual(Keyword("auto".to_string()));
        let default = Value::Numeric(NumericValue::Number(0.0));
        let mut width = &style
            .get("width")
            .cloned()
            .unwrap_or_else(|| auto.clone());
        let mut margin_left = style
            .get_fallback(&["margin", "margin-left"])
            .unwrap_or(&default);
        let mut margin_right = style
            .get_fallback(&["margin", "margin-right"])
            .unwrap_or(&default);
        let border_left = style
            .get_fallback(&["border", "border-width", "border-left"])
            .unwrap_or(&default);
        let border_right = style
            .get_fallback(&["border", "border-width", "border-right"])
            .unwrap_or(&default);
        let padding_left = style
            .get_fallback(&["padding", "padding-left"])
            .unwrap_or(&default);
        let padding_right = style
            .get_fallback(&["padding", "padding-right"])
            .unwrap_or(&default);
        let total_width: usize = [
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_px().unwrap_or(0))
        .sum();
        if width != &auto && total_width > container.content.width {
            if margin_left == &auto {
                margin_left = &default;
            }
            if margin_right == &auto {
                margin_right = &default;
            }
        }
        let underflow = container.content.width as isize - total_width as isize;
        // These values must be created outside the match so they live long enough
        let underflow_val = Value::Numeric(NumericValue::Number(underflow as f64));
        let adjusted_margin_right = Value::Numeric(NumericValue::Number(
            (margin_right.to_px().unwrap_or(0) as isize + underflow) as f64,
        ));
        let half_underflow = Value::Numeric(NumericValue::Number(underflow as f64 / 2.0));
        match (
            width == &auto,
            margin_left == &auto,
            margin_right == &auto,
        ) {
            (false, false, false) => margin_right = &adjusted_margin_right,
            (false, false, true) => margin_right = &underflow_val,
            (false, true, false) => margin_left = &underflow_val,
            (true, _, _) => {
                if underflow >= 0 {
                    width = &underflow_val
                } else {
                    width = &default;
                    margin_right = &adjusted_margin_right;
                }
            }
            (false, true, true) => {
                margin_left = &half_underflow;
                margin_right = &half_underflow;
            }
        }
        let dim = &mut self.dimensions;
        dim.content.width = width.to_px().unwrap();
        dim.padding.left = padding_left.to_px().unwrap();
        dim.padding.right = padding_right.to_px().unwrap();
        dim.border.left = border_left.to_px().unwrap();
        dim.border.right = border_right.to_px().unwrap();
        dim.margin.left = margin_left.to_px().unwrap();
        dim.margin.right = margin_right.to_px().unwrap();
    }
    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = &self.style;
        let dim = &mut self.dimensions;
        let zero = Value::Numeric(NumericValue::Number(0.0));
        dim.margin.top = style
            .get_fallback(&["margin-top", "margin"])
            .unwrap_or(&zero)
            .to_px()
            .unwrap();
        dim.margin.bottom = style
            .get_fallback(&["margin-bottom", "margin"])
            .unwrap_or(&zero)
            .to_px()
            .unwrap();
        dim.border.top = style
            .get_fallback(&["border-top-width", "border-width"])
            .unwrap_or(&zero)
            .to_px()
            .unwrap();
        dim.border.bottom = style
            .get_fallback(&["border-bottom-width", "border-width"])
            .unwrap_or(&zero)
            .to_px()
            .unwrap();
        dim.padding.top = style
            .get_fallback(&["padding-top", "padding"])
            .unwrap_or(&zero)
            .to_px()
            .unwrap();
        dim.padding.bottom = style
            .get_fallback(&["padding-top", "padding"])
            .unwrap_or(&zero)
            .to_px()
            .unwrap();
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
        if let Some(n) = self.style.get("height").map(|v| v.to_px().unwrap_or(0)) {
            self.dimensions.content.height = n as usize;
        }
    }
}

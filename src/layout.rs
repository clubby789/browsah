#![allow(dead_code, unused_variables)]

use crate::css::{NumericValue, Value};
use crate::style::{StyleMap, StyledElement};
use std::str::FromStr;

#[derive(Debug)]
pub struct LayoutBox {
    // position: Position,
    dimensions: Dimensions,
    box_type: BoxType,
    contents: Vec<LayoutBox>,
    style: StyleMap,
}

#[derive(Copy, Clone, Debug)]
struct Dimensions {
    content: Rect,
    margin: EdgeSizes,
    border: EdgeSizes,
    padding: EdgeSizes,
}

#[derive(Copy, Clone, Debug)]
enum BoxType {
    Block,
    Inline,
    Anonymous,
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
struct Rect {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct EdgeSizes {
    pub left: usize,
    pub right: usize,
    pub top: usize,
    pub bottom: usize,
}

#[derive(Copy, Clone, Debug)]
struct Position {
    x: usize,
    y: usize,
}

pub fn create_layout(root: &StyledElement, viewport_size: (usize, usize)) -> LayoutBox {
    todo!()
}

impl LayoutBox {
    fn layout(&mut self, container: Dimensions) {
        match self.box_type {
            BoxType::Block => self.layout_block(container),
            _ => todo!(),
        }
    }

    fn layout_block(&mut self, container: Dimensions) {
        self.calculate_block_width(container);
    }

    fn calculate_block_width(&mut self, container: Dimensions) {
        let style = &self.style;
        let default = Value::Numeric(NumericValue::Number(0.0));
        let mut width = &style
            .get("width")
            .cloned()
            .unwrap_or_else(|| default.clone());
        let mut margin_left = style
            .get_fallback(&["margin", "margin-left"])
            .unwrap_or(&default);
        let mut margin_right = style
            .get_fallback(&["margin", "margin-right"])
            .unwrap_or(&default);
        let border_left = style
            .get_fallback(&["border", "border-left"])
            .unwrap_or(&default);
        let border_right = style
            .get_fallback(&["border", "border-right"])
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
        let underflow = container.content.width as isize - total_width as isize;
        // These values must be created outside the match so they live long enough
        let underflow_val = Value::Numeric(NumericValue::Number(underflow as f64));
        let adjusted_margin_right = Value::Numeric(NumericValue::Number(
            (margin_right.to_px().unwrap_or(0) as isize + underflow) as f64,
        ));
        let half_underflow = Value::Numeric(NumericValue::Number(underflow as f64 / 2.0));
        match (
            width == &default,
            margin_left == &default,
            margin_right == &default,
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
}

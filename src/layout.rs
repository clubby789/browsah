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
    margin: Rect,
    border: Rect,
    padding: Rect,
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
        let width = &style
            .get("width")
            .map(|w| w.clone())
            .unwrap_or(default.clone());
        let margin_left = style
            .get_fallback(&["margin", "margin-left"])
            .unwrap_or(&default);
        let margin_right = style
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
        todo!()
    }
}

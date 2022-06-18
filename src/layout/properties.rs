use crate::style::StyleMap;
use css::{ColorValue, Value};

/// Takes a `padding`, and converts it to
/// (`padding-top`, `padding-right`, `padding-bottom`, `padding-left`)
fn to_padding_sizes<'a>(
    padding: &Value<'a>,
) -> Option<(Value<'a>, Value<'a>, Value<'a>, Value<'a>)> {
    match padding {
        Value::Number(..) | Value::Length(..) => Some((
            padding.clone(),
            padding.clone(),
            padding.clone(),
            padding.clone(),
        )),
        Value::Keyword(kw) if kw == &"auto".to_string() => Some((
            padding.clone(),
            padding.clone(),
            padding.clone(),
            padding.clone(),
        )),
        Value::Multiple(values) => {
            // Must be a space seperated value
            if !values.is_space_separated() {
                return None;
            }
            match values.0.len() {
                0 => unreachable!("Multi-value with no values"),
                1 => {
                    let v = &values.0[0].1;
                    Some((v.clone(), v.clone(), v.clone(), v.clone()))
                }
                2 => {
                    let (top, left): (&Value, &Value) = (&values.0[0].1, &values.0[1].1);
                    Some((top.clone(), left.clone(), top.clone(), left.clone()))
                }
                3 => {
                    let (top, left, bottom): (&Value, &Value, &Value) =
                        (&values.0[0].1, &values.0[1].1, &values.0[2].1);
                    Some((top.clone(), left.clone(), bottom.clone(), left.clone()))
                }
                4 => {
                    let (top, right, bottom, left): (&Value, &Value, &Value, &Value) = (
                        &values.0[0].1,
                        &values.0[1].1,
                        &values.0[2].1,
                        &values.0[3].1,
                    );
                    Some((top.clone(), right.clone(), bottom.clone(), left.clone()))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Takes a `margin`, and converts it to
/// (`margin-top`, `margin-right`, `margin-bottom`, `margin-left`)
fn to_margin_sizes<'a>(margin: &Value<'a>) -> Option<(Value<'a>, Value<'a>, Value<'a>, Value<'a>)> {
    // Same logic as padding
    to_padding_sizes(margin)
}

pub struct Padding<'a> {
    pub top: Value<'a>,
    pub right: Value<'a>,
    pub bottom: Value<'a>,
    pub left: Value<'a>,
}
impl<'a> Padding<'a> {
    pub fn new(v: (Value<'a>, Value<'a>, Value<'a>, Value<'a>)) -> Self {
        Self {
            top: v.0,
            right: v.1,
            bottom: v.2,
            left: v.3,
        }
    }
}
pub fn get_padding<'a>(style: &'a StyleMap) -> Padding<'a> {
    let mut padding = Padding::new(style.get("padding").and_then(to_padding_sizes).unwrap_or((
        Value::Number(0.0),
        Value::Number(0.0),
        Value::Number(0.0),
        Value::Number(0.0),
    )));
    if let Some(pt) = style.get("padding-top") {
        padding.top = pt.clone();
    }
    if let Some(pr) = style.get("padding-right") {
        padding.right = pr.clone();
    }
    if let Some(pb) = style.get("padding-bottom") {
        padding.bottom = pb.clone();
    }
    if let Some(pl) = style.get("padding-left") {
        padding.left = pl.clone();
    }
    padding
}

pub struct Margin<'a> {
    pub top: Value<'a>,
    pub right: Value<'a>,
    pub bottom: Value<'a>,
    pub left: Value<'a>,
}
impl<'a> Margin<'a> {
    pub fn new(v: (Value<'a>, Value<'a>, Value<'a>, Value<'a>)) -> Self {
        Self {
            top: v.0,
            right: v.1,
            bottom: v.2,
            left: v.3,
        }
    }
}
pub fn get_margins<'a>(style: &'a StyleMap) -> Margin<'a> {
    let mut margin = Margin::new(style.get("margin").and_then(to_margin_sizes).unwrap_or((
        Value::Number(0.0),
        Value::Number(0.0),
        Value::Number(0.0),
        Value::Number(0.0),
    )));
    if let Some(mt) = style.get("margin-top") {
        margin.top = mt.clone();
    }
    if let Some(mr) = style.get("margin-right") {
        margin.right = mr.clone();
    }
    if let Some(mb) = style.get("margin-bottom") {
        margin.bottom = mb.clone();
    }
    if let Some(ml) = style.get("margin-left") {
        margin.left = ml.clone();
    }
    margin
}

#[derive(Default)]
pub struct Border<'a> {
    pub left: BorderSide<'a>,
    pub right: BorderSide<'a>,
    pub top: BorderSide<'a>,
    pub bottom: BorderSide<'a>,
}
pub struct BorderSide<'a> {
    pub width: Value<'a>,
    pub style: Value<'a>,
    pub color: Value<'a>,
}

impl Default for BorderSide<'_> {
    fn default() -> Self {
        // Placeholders
        Self {
            width: Value::Number(0.0),
            style: Value::Number(0.0),
            color: Value::Color(ColorValue::black()),
        }
    }
}

/// Constructs a [`Border`] from the properties:
/// * `border`, `border-<side>`, `border-<side>-<width>`
pub fn get_border<'a>(style: &'a StyleMap) -> Border<'a> {
    let mut border = Border::default();

    if let Some(val) = style.get("border") {
        let (width, border_style, color) = process_border(val);
        if let Some(width) = width {
            border.left.width = width.clone();
            border.right.width = width.clone();
            border.bottom.width = width.clone();
            border.top.width = width;
        }
        if let Some(style) = border_style {
            border.left.style = style.clone();
            border.right.style = style.clone();
            border.bottom.style = style.clone();
            border.top.style = style;
        }
        if let Some(color) = color {
            border.left.color = color.clone();
            border.right.color = color.clone();
            border.bottom.color = color.clone();
            border.top.color = color;
        }
    }
    if let Some(val) = style.get("border-left") {
        border.left = to_border_side(val);
    }
    if let Some(val) = style.get("border-right") {
        border.right = to_border_side(val);
    }
    if let Some(val) = style.get("border-top") {
        border.top = to_border_side(val);
    }
    if let Some(val) = style.get("border-bottom") {
        border.bottom = to_border_side(val);
    }

    border
}

/// Takes a `border-<side>` and returns a [`BorderSide`]
fn to_border_side<'a>(val: &'a Value) -> BorderSide<'a> {
    match val {
        Value::Multiple(mv) => {
            if !mv.is_space_separated() {
                BorderSide::default()
            } else {
                match &mv.0.iter().map(|(_, v)| v).collect::<Vec<&Value>>()[..] {
                    &[] => unreachable!(),
                    &[width] => BorderSide {
                        width: width.clone(),
                        ..Default::default()
                    },
                    &[width, style] => BorderSide {
                        width: width.clone(),
                        style: style.clone(),
                        ..Default::default()
                    },
                    &[width, style, color] | &[width, style, color, ..] => BorderSide {
                        width: width.clone(),
                        style: style.clone(),
                        color: color.clone(),
                    },
                }
            }
        }
        _ => BorderSide {
            width: val.clone(),
            ..Default::default()
        },
    }
}

/// Tries to extract a `border-width`, `border-style` and `border-color` from the `border` property
/// These can appear in any order
fn process_border<'a>(val: &'a Value) -> (Option<Value<'a>>, Option<Value<'a>>, Option<Value<'a>>) {
    if let Value::Multiple(mv) = val {
        if !mv.is_space_separated() {
            return (None, None, None);
        }
        let values: Vec<&Value> = mv.0.iter().map(|(_, val)| val).collect();

        let width = { values.iter().cloned().cloned().find(Value::is_width) };
        let style = { values.iter().cloned().cloned().find(Value::is_border_style) };
        let color = { values.iter().cloned().cloned().find(Value::is_color) };
        (width, style, color)
    } else {
        (None, None, None)
    }
}

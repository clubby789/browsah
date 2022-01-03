use crate::style::StyleMap;
use css::Value;

/// Takes a `padding`, and converts it to
/// (`padding-top`, `padding-right`, `padding-bottom`, `padding-left`)
fn to_padding_sizes(padding: &Value) -> Option<(Value, Value, Value, Value)> {
    match padding {
        Value::Number(v) | Value::Length(v, _) => Some((
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
                0..=1 => unreachable!("Multi-value with zero or one values"),
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
fn to_margin_sizes(margin: &Value) -> Option<(Value, Value, Value, Value)> {
    // Same logic as padding
    to_padding_sizes(margin)
}

/// Takes a `border-width` and converts it to
/// (`border-top`, `border-right`, `border-bottom`, `border-left`)
fn to_border_sizes(border: &Value) -> Option<(Value, Value, Value, Value)> {
    // Same logic as padding
    to_padding_sizes(border)
}

pub struct Padding {
    pub top: Value,
    pub right: Value,
    pub bottom: Value,
    pub left: Value,
}
impl Padding {
    pub fn new(v: (Value, Value, Value, Value)) -> Self {
        Self {
            top: v.0,
            right: v.1,
            bottom: v.2,
            left: v.3,
        }
    }
}
pub fn get_padding(style: &StyleMap) -> Padding {
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

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Margin {
    pub top: Value,
    pub right: Value,
    pub bottom: Value,
    pub left: Value,
}
impl Margin {
    pub fn new(v: (Value, Value, Value, Value)) -> Self {
        Self {
            top: v.0,
            right: v.1,
            bottom: v.2,
            left: v.3,
        }
    }
}
pub fn get_margins(style: &StyleMap) -> Margin {
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
pub struct Border {
    pub left: BorderSide,
    pub right: BorderSide,
    pub top: BorderSide,
    pub bottom: BorderSide,
}
pub struct BorderSide {
    pub width: Value,
    pub style: Value,
    pub color: Value,
}

impl Default for BorderSide {
    fn default() -> Self {
        // Placeholders
        Self {
            width: Value::Number(0.0),
            style: Value::Number(0.0),
            color: Value::Number(0.0),
        }
    }
}

/// Constructs a [`Border`] from the properties:
/// * `border`, `border-<side>`, `border-<side>-<width>`
pub fn get_border(style: &StyleMap) -> Border {
    let mut border = Border::default();

    if let Some(val) = style.get("border") {
        if let Some((top, right, bottom, left)) = to_border_sizes(val) {
            border.top.width = top;
            border.right.width = right;
            border.bottom.width = bottom;
            border.left.width = left;
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
fn to_border_side(val: &Value) -> BorderSide {
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

use crate::css::{Operator, Value};

/// Takes a `padding`, and converts it to
/// (`padding-top`, `padding-right`, `padding-bottom`, `padding-left`)
pub fn to_padding_sizes(padding: &Value) -> Option<(Value, Value, Value, Value)> {
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
            if !values.0.iter().all(|v| {
                if let Some(op) = v.0 {
                    matches!(op, Operator::Space)
                } else {
                    true
                }
            }) {
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
pub fn to_margin_sizes(margin: &Value) -> Option<(Value, Value, Value, Value)> {
    // Same logic as padding
    to_padding_sizes(margin)
}

pub fn to_border_size(border: &Value) -> Option<Value> {
    match border {
        Value::Number(v) | Value::Length(v, _) => Some(border.clone()),
        Value::Multiple(mv) => {
            if !mv.0.iter().all(|(op, _)| {
                if let Some(op) = op {
                    matches!(op, Operator::Space)
                } else {
                    true
                }
            }) {
                return None;
            }
            mv.0.iter()
                .filter_map(|(_, v)| {
                    if matches!(v, Value::Number(_)) || matches!(v, Value::Length(_, _)) {
                        Some(v)
                    } else {
                        None
                    }
                })
                .next()
                .cloned()
        }
        _ => None,
    }
}

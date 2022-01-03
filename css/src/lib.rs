#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Stylesheet {
    pub rules: Vec<Ruleset>,
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Ruleset {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Selector {
    Simple(SimpleSelector),
    Compound(Vec<SimpleSelector>),
    Combinator(Box<Selector>, Combinator, Box<Selector>),
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Combinator {
    // ( )
    Descendant,
    // (>)
    Child,
    // (+)
    NextSibling,
    // (~)
    SubsequentSibling,
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SimpleSelector {
    Type(String),
    Universal,
    Attribute(String),
    Class(String),
    PseudoClass(String),
    ID(String),
}

#[macro_export]
macro_rules! simple_selector {
    (#$x:expr) => {
        SimpleSelector::ID(stringify!($x).to_string())
    };
    (.$x:expr) => {
        SimpleSelector::Class(stringify!($x).to_string())
    };
    (:$x:expr) => {
        SimpleSelector::PseudoClass(stringify!($x).to_string())
    };
    (*) => {
        SimpleSelector::Universal
    };
    ($x:expr) => {
        SimpleSelector::Type(stringify!($x).to_string())
    };
}

#[macro_export]
macro_rules! compound_selector {
    ($($sel:expr),*) => {Selector::Compound(vec![$($sel),*])}
}

#[macro_export]
macro_rules! combinator_selector {
    ($l:expr,$c:expr,$r:expr) => {
        Selector::Combinator($l.into(), $c, $r.into())
    };
}

// [att]
#[allow(dead_code)]
#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum AttributeSelector {
    // [att]
    Has(String),
    // [att=val]
    Equals(String, String),
    // [att=val1 val2]
    EqualsMany(String, Vec<String>),
    // [att|=val]
    // `att` begins with val-
    Begins(String, String),
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[allow(dead_code)]
impl Declaration {
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Value {
    Keyword(String),
    String(String),
    Url(String),
    Number(f64),
    Percentage(f64),
    Length(f64, Unit),
    Color(ColorValue),
    Function(FunctionValue),
    Multiple(MultiValue),
}

#[allow(dead_code)]
impl Value {
    /// Attempts to convert this value to a concrete pixel size
    pub fn to_px(&self, font_size: f64) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Length(n, Unit::Px) => Some(*n),
            Value::Length(n, Unit::Em) => Some(*n * font_size),
            Value::Percentage(n) => Some(*n * font_size),
            _ => None,
        }
    }
    /// Checks if this is a valid value for a 'width' (i.e. a [`Value::Number`], [`Value::Length`]
    /// or a valid keyword
    pub fn is_width(&self) -> bool {
        if let Value::Keyword(kw) = self {
            ["thin", "medium", "thick"].contains(&kw.as_str())
        } else {
            matches!(self, Value::Number(..)) || matches!(self, Value::Length(..))
        }
    }
    /// Checks if this is a valid border-style keyword
    pub fn is_border_style(&self) -> bool {
        if let Value::Keyword(kw) = self {
            [
                "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge",
                "inset", "outset",
            ]
            .contains(&kw.as_str())
        } else {
            false
        }
    }
    /// Check if this is a valid color, color-function or color-keyword
    pub fn is_color(&self) -> bool {
        match self {
            // TODO: Color keywords
            Value::Keyword(_) => false,
            Value::Color(_) => true,
            Value::Function(func) => {
                ["rgb", "rgba", "hsl", "hsla", "hwb"].contains(&func.0.as_str())
            }
            _ => false,
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Unit {
    Cm,
    Mm,
    Q,
    In,
    Pc,
    Pt,
    Px,
    Em,
    Ex,
    Ch,
    Rem,
    Lh,
    Vw,
    Vh,
    Vmin,
    Vmax,
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ColorValue {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorValue {
    pub fn new(val: &[u8]) -> Self {
        if let [r, g, b, a] = val {
            Self {
                r: *r,
                g: *g,
                b: *b,
                a: *a,
            }
        } else {
            unreachable!()
        }
    }
    pub const fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
    pub const fn white() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

pub fn interpolate_color(from: ColorValue, to: ColorValue, fac: f32) -> ColorValue {
    if from == to {
        return from;
    }
    let r = interpolate_val(from.r, to.r, fac);
    let g = interpolate_val(from.g, to.g, fac);
    let b = interpolate_val(from.b, to.b, fac);
    let a = interpolate_val(from.a, to.a, fac);
    ColorValue { r, g, b, a }
}

fn interpolate_val(from: u8, to: u8, fac: f32) -> u8 {
    use std::cmp::Ordering;
    match from.cmp(&to) {
        Ordering::Equal => from,
        Ordering::Less => to - ((to - from) as f32 * fac) as u8,
        Ordering::Greater => from - ((from - to) as f32 * fac) as u8,
    }
}

#[cfg(test)]
#[test]
fn test_interpolate() {
    let white = ColorValue::new(&[255, 255, 255, 255]);
    let black = ColorValue::new(&[0, 0, 0, 255]);

    assert_eq!(interpolate_color(white, black, 1.0), black);
    assert_eq!(
        interpolate_color(white, black, 0.5),
        ColorValue::new(&[128, 128, 128, 255])
    );
    assert_eq!(interpolate_color(white, black, 0.0), white);
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct MultiValue(pub Vec<(Option<Operator>, Value)>);

impl MultiValue {
    pub fn is_space_separated(&self) -> bool {
        self.0.iter().all(|v| {
            if let Some(op) = v.0 {
                matches!(op, Operator::Space)
            } else {
                true
            }
        })
    }
    pub fn new_space_seperated(values: &[&Value]) -> Self {
        let mut v = Vec::with_capacity(values.len());
        v.push((None, values[0].clone()));
        v.extend(
            values[1..]
                .iter()
                .map(|&val| (Some(Operator::Space), val.clone())),
        );
        Self(v)
    }
}

#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[repr(u8)]
pub enum Operator {
    Slash = b'/',
    Comma = b',',
    Space = b' ',
    Equals = b'=',
}

#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FunctionValue(String, Vec<Value>);

mod keywords;

pub use keywords::*;

pub fn keyword_to_value(kw: String) -> Option<Value> {
    match kw.as_str() {
        "black" => Some(Value::Color(BLACK)),
        _ => None,
    }
}
pub fn function_to_value(func: FunctionValue) -> Option<Value> {
    match func.0.as_str() {
        "rgb" => {
            let mut args: Vec<u8> = func
                .1
                .iter()
                .take(3)
                .map(|v| {
                    if let Value::Number(val) = v {
                        *val as u8
                    } else {
                        unreachable!()
                    }
                })
                .collect();
            args.push(255);
            Some(Value::Color(ColorValue::new(args.as_slice())))
        }
        "rgba" => {
            let args: Vec<u8> = func
                .1
                .iter()
                .take(4)
                .map(|v| {
                    if let Value::Number(val) = v {
                        *val as u8
                    } else {
                        unreachable!()
                    }
                })
                .collect();
            Some(Value::Color(ColorValue::new(args.as_slice())))
        }
        _ => None,
    }
}

mod parsing;
#[cfg(test)]
mod tests;

pub use parsing::stylesheet;

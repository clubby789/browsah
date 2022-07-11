#[derive(PartialEq, Clone, Debug)]
pub struct Stylesheet<'a> {
    pub rules: Vec<Ruleset<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Ruleset<'a> {
    pub selectors: Vec<Selector<'a>>,
    pub declarations: Vec<Declaration<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Selector<'a> {
    Simple(SimpleSelector<'a>),
    Compound(Vec<SimpleSelector<'a>>),
    Combinator(Box<Selector<'a>>, Combinator, Box<Selector<'a>>),
}

#[derive(PartialEq, Clone, Copy, Debug)]
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

#[derive(PartialEq, Clone, Debug)]
pub enum SimpleSelector<'a> {
    Type(&'a str),
    Universal,
    Attribute(&'a str),
    Class(&'a str),
    PseudoClass(&'a str),
    ID(&'a str),
}

#[macro_export]
macro_rules! simple_selector {
    (#$x:expr) => {
        SimpleSelector::ID(stringify!($x))
    };
    (.$x:expr) => {
        SimpleSelector::Class(stringify!($x))
    };
    (:$x:expr) => {
        SimpleSelector::PseudoClass(stringify!($x))
    };
    (*) => {
        SimpleSelector::Universal
    };
    ($x:expr) => {
        SimpleSelector::Type(stringify!($x))
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
#[derive(PartialEq, Clone, Debug)]
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

#[derive(PartialEq, Clone, Debug)]
pub struct Declaration<'a> {
    pub name: &'a str,
    pub value: Value<'a>,
}

#[allow(dead_code)]
impl<'a> Declaration<'a> {
    pub fn new(name: &'a str, value: Value<'a>) -> Self {
        Self { name, value }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Debug)]
pub enum Value<'a> {
    Keyword(&'a str),
    String(&'a str),
    Url(&'a str),
    Number(f64),
    Percentage(f64),
    Length(f64, Unit),
    Color(ColorValue),
    Function(FunctionValue<'a>),
    Multiple(MultiValue<'a>),
}

impl<'a> Value<'a> {
    /// Attempts to convert this value to a concrete pixel size
    pub fn try_to_px(&self, font_size: f64) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Length(n, Unit::Px) => Some(*n),
            Value::Length(n, Unit::Em) => Some(*n * font_size),
            Value::Percentage(n) => Some(*n * font_size),
            _ => None,
        }
    }
    /// Attempts this valid to a color
    pub fn try_to_color(&self) -> Option<ColorValue> {
        match self {
            Value::Keyword(_) => None,
            Value::Color(col) => Some(*col),
            Value::Function(_) => None,
            _ => None,
        }
    }
    /// Checks if this is a valid value for a 'width' (i.e. a [`Value::Number`], [`Value::Length`]
    /// or a valid keyword
    pub fn is_width(&self) -> bool {
        if let Value::Keyword(kw) = self {
            ["thin", "medium", "thick"].contains(kw)
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
            .contains(kw)
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
            Value::Function(func) => ["rgb", "rgba", "hsl", "hsla", "hwb"].contains(&func.0),
            _ => false,
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy, Debug)]
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

#[derive(PartialEq, Clone, Copy, Debug)]
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

#[derive(PartialEq, Clone, Debug)]
pub struct MultiValue<'a>(pub Vec<(Option<Operator>, Value<'a>)>);

impl<'a> MultiValue<'a> {
    pub fn is_space_separated(&self) -> bool {
        self.0.iter().all(|v| {
            if let Some(op) = v.0 {
                matches!(op, Operator::Space)
            } else {
                true
            }
        })
    }
    pub fn new_space_seperated(values: &[&Value<'a>]) -> Self {
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

#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum Operator {
    Slash = b'/',
    Comma = b',',
    Space = b' ',
    Equals = b'=',
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionValue<'a>(&'a str, Vec<Value<'a>>);

mod keywords;

pub use keywords::*;

/// Takes a CSS keyword and returns a Value. If the keyword is implemented,
/// the proper value will be returned. Otherwise, it will be returned as a Value::Keyword
pub fn keyword_to_value(kw: &str) -> Value {
    match kw {
        "black" => Value::Color(BLACK),
        _ => Value::Keyword(kw),
    }
}

/// Takes a CSS function call and returns a Value. If the function is implemented,
/// the proper value will be returned. Otherwise, it will be returned as a Value::FunctionValue
pub fn function_to_value(func: FunctionValue) -> Value {
    match func.0 {
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
            Value::Color(ColorValue::new(args.as_slice()))
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
            Value::Color(ColorValue::new(args.as_slice()))
        }
        _ => Value::Function(func),
    }
}

mod parsing;
#[cfg(test)]
mod tests;

pub use parsing::stylesheet;

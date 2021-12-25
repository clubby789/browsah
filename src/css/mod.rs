#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone)]
pub struct Stylesheet {
    rules: Vec<Ruleset>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ruleset {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Selector {
    Simple(SimpleSelector),
    Compound(Vec<SimpleSelector>),
    Combinator(Box<Selector>, Combinator, Box<Selector>),
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum SimpleSelector {
    Type(TypeSelector),
    Universal,
    Attribute(AttributeSelector),
    Class(ClassSelector),
    PseudoClass(PseudoClassSelector),
    ID(IDSelector),
}

#[allow(unused_macros)]
macro_rules! simple_selector {
    (#$x:expr) => {
        SimpleSelector::ID(IDSelector(stringify!($x).to_string()))
    };
    (.$x:expr) => {
        SimpleSelector::Class(ClassSelector(stringify!($x).to_string()))
    };
    (:$x:expr) => {
        SimpleSelector::PseudoClass(PseudoClassSelector(stringify!($x).to_string()))
    };
    (*) => {
        SimpleSelector::Universal
    };
    ($x:expr) => {
        SimpleSelector::Type(TypeSelector(stringify!($x).to_string()))
    };
}

// h1
#[derive(Debug, PartialEq, Clone)]
pub struct TypeSelector(String);
// [att]
#[derive(Debug, PartialEq, Clone)]
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
// .class
#[derive(Debug, PartialEq, Clone)]
pub struct ClassSelector(String);
// #id
#[derive(Debug, PartialEq, Clone)]
pub struct IDSelector(String);
// :valid
#[derive(Debug, PartialEq, Clone)]
pub struct PseudoClassSelector(String);

#[derive(Debug, PartialEq, Clone)]
pub struct Declaration {
    name: String,
    value: Value,
}

impl Declaration {
    pub fn new(name: impl Into<String>, value: Value) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Textual(TextValue),
    Numeric(NumericValue),
    Dimension(NumericValue, Unit),
    Color(ColorValue),
    Function(FunctionValue),
    Multiple(MultiValue),
    Image,
    Position,
}

impl Value {
    pub fn textual(v: impl Into<TextValue>) -> Self {
        Self::Textual(v.into())
    }
    pub fn numeric(v: impl Into<NumericValue>) -> Self {
        Self::Numeric(v.into())
    }
    pub fn dimension(v: impl Into<NumericValue>, u: impl Into<Unit>) -> Self {
        Self::Dimension(v.into(), u.into())
    }
    pub fn color(v: impl Into<ColorValue>) -> Self {
        Self::Color(v.into())
    }
    pub fn function(v: impl Into<FunctionValue>) -> Self {
        Self::Function(v.into())
    }
    pub fn multiple(v: impl Into<MultiValue>) -> Self {
        Self::Multiple(v.into())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TextValue {
    Keyword(String),
    String(String),
    Url(String),
}

impl TextValue {
    pub fn keyword(v: impl Into<String>) -> Self {
        Self::Keyword(v.into())
    }
    pub fn string(v: impl Into<String>) -> Self {
        Self::String(v.into())
    }
    pub fn url(v: impl Into<String>) -> Self {
        Self::Url(v.into())
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum NumericValue {
    Integer(isize),
    Number(f64),
    Percentage(f64),
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ColorValue {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct MultiValue(Vec<(Option<Operator>, Value)>);

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Operator {
    Slash = b'/',
    Comma = b',',
    Space = b' ',
    Equals = b'=',
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionValue(String, Vec<Value>);

mod keywords;

use keywords::*;

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
                .map(|v| {
                    if let Value::Numeric(NumericValue::Number(val)) = v {
                        *val as u8
                    } else {
                        unreachable!()
                    }
                })
                .collect();
            args.push(255);
            Some(Value::Color(ColorValue::new(args.as_slice())))
        }
        _ => None,
    }
}

mod parsing;
#[cfg(test)]
mod tests;

pub use parsing::stylesheet;

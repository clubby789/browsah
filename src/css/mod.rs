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

// TODO: Font value??
// Do other values look like identifier, identifier?
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Keyword(String),
    Measurement(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    Px,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

mod spec;

#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    rules: Vec<Rule>,
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq)]
pub enum Selector {
    Universal,
    Simple(SimpleSelector),
}

#[derive(Debug, PartialEq, Default)]
pub struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct Declaration {
    name: String,
    value: Value,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Keyword(String),
    Measurement(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug, PartialEq)]
pub enum Unit {
    Px,
}

#[derive(Debug, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

mod parsing;

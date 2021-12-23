#[derive(Debug, PartialEq)]
struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, PartialEq)]
struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, PartialEq)]
enum Selector {
    Universal,
    Simple(SimpleSelector),
}

#[derive(Debug, PartialEq, Default)]
struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Option<String>,
}

#[derive(Debug, PartialEq)]
struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
enum Value {
    Keyword(String),
    Measurement(f32, Unit),
    ColorValue(Color),
}

#[derive(Debug, PartialEq)]
enum Unit {
    Px,
}

#[derive(Debug, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub mod parsing;

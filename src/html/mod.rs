use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DOMElement {
    pub name: String,
    pub attributes: DOMAttributes,
    pub contents: Vec<DOMContent>,
}

impl DOMElement {
    pub fn new(
        name: impl Into<String>,
        attributes: Option<DOMAttributes>,
        contents: Vec<DOMContent>,
    ) -> Self {
        Self {
            name: name.into().to_lowercase(),
            attributes: attributes.unwrap_or_default(),
            contents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DOMAttributes(HashMap<String, String>);

#[allow(unused_macros)]
macro_rules! attributes {
    ($($name:expr => $value:expr),*) => {
        DOMAttributes(HashMap::from([
            $((stringify!($name).replace(" ", ""), stringify!($value).replace(" ", "")))*
        ]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DOMContent {
    Text(String),
    Element(DOMElement),
}
impl From<String> for DOMContent {
    fn from(s: String) -> DOMContent {
        DOMContent::Text(s)
    }
}
impl From<&str> for DOMContent {
    fn from(s: &str) -> DOMContent {
        DOMContent::Text(s.into())
    }
}
impl From<DOMElement> for DOMContent {
    fn from(e: DOMElement) -> DOMContent {
        DOMContent::Element(e)
    }
}

mod parsing;
#[cfg(test)]
mod tests;

pub use parsing::document;

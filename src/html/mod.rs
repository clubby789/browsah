use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DOMElement {
    name: String,
    attributes: DOMAttributes,
    contents: Vec<DOMContent>,
}

impl DOMElement {
    pub fn void(name: impl Into<String>, attributes: Option<DOMAttributes>) -> Self {
        Self {
            name: name.into().to_lowercase(),
            attributes: attributes.unwrap_or_default(),
            contents: Default::default(),
        }
    }
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
impl Into<DOMContent> for String {
    fn into(self) -> DOMContent {
        DOMContent::Text(self)
    }
}
impl Into<DOMContent> for &str {
    fn into(self) -> DOMContent {
        DOMContent::Text(self.into())
    }
}
impl Into<DOMContent> for DOMElement {
    fn into(self) -> DOMContent {
        DOMContent::Element(self)
    }
}

mod parsing;
#[cfg(test)]
mod tests;

pub use parsing::document;

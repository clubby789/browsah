#![feature(iter_intersperse)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct DOMElement {
    pub name: String,
    pub attributes: DOMAttributes,
    pub contents: Vec<DOMContent>,
}

impl DOMElement {
    pub fn new(name: &str, attributes: Option<DOMAttributes>, contents: Vec<DOMContent>) -> Self {
        Self {
            name: name.to_lowercase(),
            attributes: attributes.unwrap_or_default(),
            contents,
        }
    }

    pub fn get_elements_by_name(
        &self,
        name: impl Into<String>,
        recursive: bool,
    ) -> Vec<&DOMElement> {
        let name = name.into();
        if !recursive {
            self.contents
                .iter()
                .filter_map(|c| {
                    if let DOMContent::Element(elt) = c {
                        if elt.name == name {
                            Some(elt)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            todo!()
        }
    }

    pub fn get_attribute(&self, name: impl Into<String>) -> Option<&String> {
        let name = name.into();
        self.attributes.0.get(name.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DOMAttributes(pub HashMap<String, String>);

#[macro_export]
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

impl From<&str> for DOMContent {
    fn from(s: &str) -> Self {
        DOMContent::Text(s.to_string())
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

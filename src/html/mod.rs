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

#[allow(unused_macros)]
macro_rules! attributes {
    ($($name:expr => $value:expr),*) => {
        DOMAttributes(HashMap::from([
            $((stringify!($name).replace(" ", ""), stringify!($value).replace(" ", "")))*
        ]))
    }
}
#[allow(unused_imports)]
pub(crate) use attributes;

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

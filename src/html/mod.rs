use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct DOMNode {
    children: Vec<DOMNode>,
    node_type: DOMNodeType,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DOMNodeType {
    Element(DOMElement),
    Text(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct DOMElement {
    tag_name: String,
    attributes: DOMAttributes,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DOMAttributes(HashMap<String, String>);

impl DOMAttributes {
    fn empty() -> Self {
        Self(HashMap::new())
    }
}

mod parsing;

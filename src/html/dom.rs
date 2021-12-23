use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq)]
pub struct DOMNode {
    pub children: Vec<DOMNode>,
    pub node_type: DOMNodeType,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DOMNodeType {
    Element(DOMElement),
    Text(String),
}

#[derive(Debug, Eq, PartialEq)]
pub struct DOMElement {
    pub tag_name: String,
    pub attributes: DOMAttributes,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DOMAttributes(pub HashMap<String, String>);

impl DOMAttributes {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }
}

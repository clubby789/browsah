use super::html::DOMElement;
use crate::css::{Ruleset, Stylesheet, Value};
use crate::html::DOMContent;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug)]
pub struct StyledElement {
    // The attribute name -> the value along with the specificity of the selector
    pub styles: HashMap<String, (Value, Specificity)>,
    pub content: Vec<StyledContent>,
}

#[derive(Debug)]
pub enum StyledContent {
    Element(StyledElement),
    Text(String),
}

impl StyledElement {
    pub fn insert(&mut self, key: String, value: Value, spec: Specificity) {
        // Insert the new declaration only if the attribute is not specified *or* the specificity is lower
        if let Some(&(_, existing)) = self.styles.get(&key) {
            if spec >= existing {
                self.styles.insert(key, (value, spec));
            };
        } else {
            self.styles.insert(key, (value, spec));
        }
    }
}

// IDs, Classes, Elements
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct Specificity(usize, usize, usize);

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let Specificity(ids, cls, elts) = self;
        let value = ids * 100 + cls * 10 + elts;
        let Specificity(ids, cls, elts) = other;
        let o_value = ids * 100 + cls * 10 + elts;
        std::cmp::PartialOrd::partial_cmp(&value, &o_value)
    }
}

#[cfg(test)]
#[test]
fn test_specificity() {
    let a = Specificity(0, 0, 1);
    let b = Specificity(1, 0, 1);
    let c = Specificity(0, 0, 0);
    assert!(b > a);
    assert!(a > c);
    assert!(b > c);
}

impl From<DOMContent> for StyledContent {
    fn from(content: DOMContent) -> Self {
        match content {
            DOMContent::Text(s) => StyledContent::Text(s),
            DOMContent::Element(e) => StyledContent::Element(e.into()),
        }
    }
}
impl From<DOMElement> for StyledElement {
    fn from(element: DOMElement) -> Self {
        Self {
            styles: Default::default(),
            content: element.contents.into_iter().map(|e| e.into()).collect(),
        }
    }
}

pub fn construct_style_tree(dom: DOMElement, css: Stylesheet) -> StyledElement {
    let mut tree: StyledElement = dom.into();
    tree.apply_styles(css.rules);
    tree
}

impl StyledElement {
    pub fn apply_styles(&mut self, styles: Vec<Ruleset>) {
        styles.iter().for_each(|r| self.apply_rule(r));
    }

    fn apply_rule(&mut self, style: &Ruleset) {
        todo!()
    }
}

use super::html::DOMElement;
use crate::css::{
    Declaration, Ruleset, Selector,
    SimpleSelector, Stylesheet, Value,
};
use crate::html::{DOMAttributes, DOMContent};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::Sum;
use std::ops::{Add, Deref};

pub type StyleMap = HashMap<String, (Value, Specificity)>;

#[derive(Debug)]
pub struct StyledElement {
    pub name: String,
    pub attributes: DOMAttributes,
    pub contents: Vec<StyledContent>,
    // The attribute name -> the value along with the specificity of the selector
    pub styles: StyleMap,
}

#[derive(Debug)]
pub struct StyledString {
    pub contents: String,
    pub styles: StyleMap,
}

impl From<&String> for StyledString {
    fn from(s: &String) -> Self {
        StyledString {
            contents: s.clone(),
            styles: Default::default(),
        }
    }
}

impl From<String> for StyledString {
    fn from(s: String) -> Self {
        StyledString {
            contents: s,
            styles: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum StyledContent {
    Element(StyledElement),
    Text(StyledString),
}

impl StyledElement {
    pub fn insert(&mut self, key: String, value: Value, spec: Specificity) {
        // Insert the new declaration only if the attribute is not specified *or*
        // the specificity is lower
        if let Some(&(_, existing)) = self.styles.get(&key) {
            if spec >= existing {
                self.styles.insert(key, (value, spec));
            };
        } else {
            self.styles.insert(key, (value, spec));
        }
    }
}

// Attrs, IDs, Classes, Elements
#[derive(PartialEq, Copy, Clone, Default, Debug, Eq)]
pub struct Specificity(usize, usize, usize, usize);

impl From<(usize, usize, usize, usize)> for Specificity {
    fn from(el: (usize, usize, usize, usize)) -> Self {
        Self(el.0, el.1, el.2, el.3)
    }
}

impl PartialOrd for Specificity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let Specificity(attrs, ids, cls, elts) = self;
        let value = attrs * 1000 + ids * 100 + cls * 10 + elts;
        let Specificity(attrs, ids, cls, elts) = other;
        let o_value = attrs * 1000 + ids * 100 + cls * 10 + elts;
        std::cmp::PartialOrd::partial_cmp(&value, &o_value)
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        let Specificity(attrs, ids, cls, elts) = self;
        let value = attrs * 1000 + ids * 100 + cls * 10 + elts;
        let Specificity(attrs, ids, cls, elts) = other;
        let o_value = attrs * 1000 + ids * 100 + cls * 10 + elts;
        std::cmp::Ord::cmp(&value, &o_value)
    }
}

#[cfg(test)]
#[test]
fn test_specificity() {
    let a = Specificity(0, 0, 0, 1);
    let b = Specificity(0, 1, 0, 1);
    let c = Specificity(0, 0, 0, 0);
    assert!(b > a);
    assert!(a > c);
    assert!(b > c);
}

impl Sum for Specificity {
    fn sum<I: Iterator<Item = Specificity>>(iter: I) -> Self {
        iter.fold((0, 0, 0, 0).into(), |acc, x| acc + x)
    }
}

impl Add<Specificity> for Specificity {
    type Output = Self;
    fn add(self, rhs: Specificity) -> Self::Output {
        Specificity(
            self.0 + rhs.0,
            self.1 + rhs.1,
            self.2 + rhs.2,
            self.3 + rhs.3,
        )
    }
}

impl From<&Selector> for Specificity {
    fn from(sel: &Selector) -> Self {
        match sel {
            Selector::Simple(sel) => sel.into(),
            Selector::Compound(sels) => sels.iter().map(|s| s.into()).sum(),
            Selector::Combinator(l, c, r) => {
                Specificity::from(l.deref()) + Specificity::from(r.deref())
            }
        }
    }
}

impl From<&SimpleSelector> for Specificity {
    fn from(sel: &SimpleSelector) -> Self {
        match sel {
            SimpleSelector::Type(_) => (0, 0, 0, 1),
            SimpleSelector::Universal => (0, 0, 0, 0),
            SimpleSelector::Attribute(_) => (1, 0, 0, 0),
            SimpleSelector::Class(_) => (0, 0, 1, 0),
            SimpleSelector::PseudoClass(_) => (0, 0, 1, 0),
            SimpleSelector::ID(_) => (0, 1, 0, 0),
        }
        .into()
    }
}

impl From<DOMContent> for StyledContent {
    fn from(content: DOMContent) -> Self {
        match content {
            DOMContent::Text(s) => StyledContent::Text(s.into()),
            DOMContent::Element(e) => StyledContent::Element(e.into()),
        }
    }
}

impl From<DOMElement> for StyledElement {
    fn from(element: DOMElement) -> Self {
        Self {
            name: element.name,
            styles: Default::default(),
            contents: element.contents.into_iter().map(|e| e.into()).collect(),
            attributes: element.attributes,
        }
    }
}

pub fn construct_style_tree(dom: DOMElement, css: Stylesheet) -> StyledElement {
    let mut tree: StyledElement = dom.into();
    tree.apply_styles(css.rules);
    tree
}

impl StyledElement {
    /// Iterate over each ruleset in a stylesheet and apply it to the DOM
    pub fn apply_styles(&mut self, styles: Vec<Ruleset>) {
        styles.iter().for_each(|r| self.apply_rule(r));
    }

    /// Find the highest specificity (if any) selector for a given node and apply it
    fn apply_rule(&mut self, style: &Ruleset) {
        if let Some(spec) = style
            .selectors
            .iter()
            .filter(|r| self.does_rule_apply(r))
            .map(Specificity::from)
            .max()
        {
            // Styles will be inherited by children
            return self.apply_rule_unconditionally(&style.declarations, spec);
        }
        // Didn't apply to the parent, so we need to check each child recursively
        for content in &mut self.contents {
            if let StyledContent::Element(elt) = content {
                elt.apply_rule(style);
            }
        }
    }

    fn does_rule_apply(&self, selector: &Selector) -> bool {
        match selector {
            Selector::Simple(s) => self.does_simple_selector_apply(s),
            Selector::Compound(sels) => sels.iter().any(|s| self.does_simple_selector_apply(s)),
            Selector::Combinator(_, _, _) => todo!(),
        }
    }

    fn does_simple_selector_apply(&self, selector: &SimpleSelector) -> bool {
        match selector {
            SimpleSelector::Type(name) => self.name == name.0,
            SimpleSelector::Universal => true,
            SimpleSelector::Attribute(_) => todo!(),
            SimpleSelector::Class(name) => self
                .attributes
                .0
                .get("class")
                .map(|c: &String| c.split_whitespace().any(|c| c == name.0.as_str()))
                .unwrap_or(false),
            SimpleSelector::PseudoClass(_) => todo!(),
            SimpleSelector::ID(_) => todo!(),
        }
    }

    /// Apply a list of [`Declaration`]s to a node and all its children recursively
    fn apply_rule_unconditionally(&mut self, declarations: &[Declaration], spec: Specificity) {
        declarations.iter().for_each(|decl| {
            let (name, value) = (decl.name.clone(), decl.value.clone());
            self.styles.insert(name, (value, spec));
        });
        self.contents.iter_mut().for_each(|content| {
            if let StyledContent::Element(elt) = content {
                elt.apply_rule_unconditionally(declarations, spec)
            }
        });
    }
}

#[cfg(test)]
#[test]
fn test_does_apply() {
    use crate::{html::{attributes}, css::{compound_selector, simple_selector, ClassSelector, TypeSelector}};
    let dom: StyledElement = DOMElement::new("div", None, vec![]).into();
    let style: Selector = Selector::Simple(simple_selector!(div));
    assert!(dom.does_rule_apply(&style));

    let dom: StyledElement = DOMElement::new("p", None, vec![]).into();
    assert!(!dom.does_rule_apply(&style));

    let style: Selector = compound_selector!(simple_selector!(div), simple_selector!(.wide));
    let dom: StyledElement = DOMElement::new("p", Some(attributes! {class=>wide}), vec![]).into();
    assert!(dom.does_rule_apply(&style));
}

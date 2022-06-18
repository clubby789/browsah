use css::{stylesheet, Declaration, Ruleset, Selector, SimpleSelector, Stylesheet, Value};
use html::{DOMAttributes, DOMContent, DOMElement};
use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::iter::Sum;
use std::ops::{Add, Deref};

static USER_AGENT_STYLESHEET: &str = include_str!("../resources/html.css");
pub static USER_AGENT_CSS: Lazy<Stylesheet> =
    Lazy::new(|| stylesheet(USER_AGENT_STYLESHEET).unwrap().1);

#[derive(Default, Clone)]
pub struct StyleMap<'a>(HashMap<&'a str, (Value<'a>, Specificity)>);

impl<'a> StyleMap<'a> {
    pub fn get(&self, value: &str) -> Option<&Value> {
        self.0.get(value).map(|v| &v.0)
    }
}

pub struct StyledElement<'a> {
    pub name: String,
    pub attributes: DOMAttributes,
    pub contents: Vec<StyledContent<'a>>,
    // The attribute name -> the value along with the specificity of the selector
    pub styles: StyleMap<'a>,
}

pub struct StyledString<'a> {
    pub contents: Cow<'a, str>,
    pub styles: StyleMap<'a>,
}

impl<'a> From<&'a str> for StyledString<'a> {
    fn from(s: &'a str) -> Self {
        StyledString {
            contents: Cow::Borrowed(s),
            styles: Default::default(),
        }
    }
}

impl<'a> From<String> for StyledString<'a> {
    fn from(s: String) -> Self {
        StyledString {
            contents: Cow::Owned(s),
            styles: Default::default(),
        }
    }
}

pub enum StyledContent<'a> {
    Element(StyledElement<'a>),
    Text(StyledString<'a>),
}

impl<'a> StyledElement<'a> {
    /// Insert a CSS declaration (key/[`Value`]) only if the [`Specificity`] of the
    /// existing rule for that key is lower (or does not exist)
    pub fn insert<'b>(&'b mut self, key: &'a str, value: Value<'a>, spec: Specificity)
    where
        'a: 'b,
    {
        // Insert the new declaration only if the attribute is not specified *or*
        // the specificity is lower
        if let Some(&(_, existing)) = self.styles.0.get(&key) {
            if spec >= existing {
                self.styles.0.insert(key, (value, spec));
            };
        } else {
            self.styles.0.insert(key, (value, spec));
        }
    }
}

// Attrs, IDs, Classes, Elements
#[derive(PartialEq, Copy, Clone, Default, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
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

impl<'a> From<&Selector<'a>> for Specificity {
    fn from(sel: &Selector) -> Self {
        match sel {
            Selector::Simple(sel) => sel.into(),
            Selector::Compound(sels) => sels.iter().map(|s| s.into()).sum(),
            Selector::Combinator(l, _, r) => {
                Specificity::from(l.deref()) + Specificity::from(r.deref())
            }
        }
    }
}

impl<'a> From<&SimpleSelector<'a>> for Specificity {
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

impl<'a> From<DOMContent> for StyledContent<'a> {
    fn from(content: DOMContent) -> Self {
        match content {
            DOMContent::Text(s) => StyledContent::Text(s.into()),
            DOMContent::Element(e) => StyledContent::Element(e.into()),
        }
    }
}

impl<'a> From<DOMElement> for StyledElement<'a> {
    fn from(element: DOMElement) -> Self {
        Self {
            name: element.name,
            styles: Default::default(),
            contents: element
                .contents
                .into_iter()
                .filter(|c| {
                    if let DOMContent::Element(elt) = c {
                        !element_is_excluded(elt)
                    } else {
                        true
                    }
                })
                .map(|e| e.into())
                .collect(),
            attributes: element.attributes,
        }
    }
}

/// Certain elements should not be copied into the style tree, i.e. do not produce a 'box'
fn element_is_excluded(elt: &DOMElement) -> bool {
    EXCLUDED.contains(&elt.name.as_str())
}

impl<'a> StyledElement<'a> {
    /// Iterate over each ruleset in a stylesheet and apply it to the DOM
    pub fn apply_styles<'b>(&'b mut self, styles: &[Ruleset<'a>])
    where
        'a: 'b,
    {
        for r in styles {
            self.apply_rule(r);
        }
    }

    /// Find the highest specificity (if any) selector for a given node and apply it
    /// If this function returns `true`, then the element is not being displayed and should
    /// be deleted by the parent
    fn apply_rule<'b>(&'b mut self, style: &Ruleset<'a>) -> bool
    where
        'a: 'b,
    {
        if let Some(spec) = style
            .selectors
            .iter()
            .filter(|r| self.does_rule_apply(r))
            .map(Specificity::from)
            .max()
        {
            if style
                .declarations
                .iter()
                .any(|decl| decl.name == "display" && decl.value == Value::Keyword("none"))
            {
                return true;
            }
            // Styles will be inherited by children
            self.apply_rule_unconditionally(&style.declarations, spec, false);
        }
        // Didn't apply to the parent, so we need to check each child recursively
        let mut remove = Vec::new();
        for (i, content) in self.contents.iter_mut().enumerate() {
            if let StyledContent::Element(elt) = content {
                if elt.apply_rule(style) {
                    remove.push(i);
                }
            }
        }
        // Iterate backwards so we can remove elements from the array
        remove.into_iter().rev().for_each(|i| {
            self.contents.remove(i);
        });
        false
    }

    /// Check if the provided [`Selector`] selects this element
    fn does_rule_apply(&self, selector: &Selector) -> bool {
        match selector {
            Selector::Simple(s) => self.does_simple_selector_apply(s),
            Selector::Compound(sels) => sels.iter().any(|s| self.does_simple_selector_apply(s)),
            // TODO: Implement combinators
            Selector::Combinator(_, _, _) => false,
        }
    }

    /// Check if the provided [`SimpleSelector`] selects this element
    fn does_simple_selector_apply(&self, selector: &SimpleSelector) -> bool {
        match selector {
            SimpleSelector::Type(name) => &self.name == name,
            SimpleSelector::Universal => true,
            // TODO: Implement
            SimpleSelector::Attribute(_) => false,
            SimpleSelector::Class(name) => self.has_class(name),
            // TODO: Implement
            SimpleSelector::PseudoClass(_) => false,
            SimpleSelector::ID(id) => self.id_is(id),
        }
    }

    /// Check if the `class` attribute is present and contains the specified class
    fn has_class(&self, class: &str) -> bool {
        let class: String = class.into();
        self.attributes
            .0
            .get("class")
            .map(|c: &String| c.split_whitespace().any(|c| *c == class))
            .unwrap_or(false)
    }

    /// Check if the `id` attribute exists and is an exact match for the provided ID
    fn id_is(&self, id: &str) -> bool {
        let id: String = id.into();
        self.attributes
            .0
            .get("id")
            .map(|c| c == &id)
            .unwrap_or(false)
    }

    /// Apply a list of [`Declaration`]s to a node and all its children recursively
    /// `inherit_all`: Whether to apply *every* declaration to children. If this is `false`, only
    /// properties specified in [value@INHERITED] will be applied recursively
    fn apply_rule_unconditionally<'b>(
        &'b mut self,
        declarations: &[Declaration<'a>],
        spec: Specificity,
        inherit_all: bool,
    ) where
        'a: 'b,
    {
        for decl in declarations {
            let (name, value) = (decl.name, decl.value.clone());
            self.insert(name, value, spec);
        }
        let inherited: Vec<Declaration> = if inherit_all {
            // If this is true, we aren't in the top level and our declarations have already been
            // filtered
            declarations.to_vec()
        } else {
            declarations
                .iter()
                .cloned()
                .filter(|d| INHERITED.contains(&d.name))
                .collect()
        };
        self.contents.iter_mut().for_each(|content| {
            if let StyledContent::Element(elt) = content {
                elt.apply_rule_unconditionally(inherited.as_slice(), spec, true)
            }
        });
    }
}

#[cfg(test)]
#[test]
fn test_does_apply() {
    use {
        css::{compound_selector, simple_selector},
        html::attributes,
    };
    let dom: StyledElement = DOMElement::new("div", None, vec![]).into();
    let style: Selector = Selector::Simple(simple_selector!(div));
    assert!(dom.does_rule_apply(&style));

    let dom: StyledElement = DOMElement::new("p", None, vec![]).into();
    assert!(!dom.does_rule_apply(&style));

    let style: Selector = compound_selector!(simple_selector!(div), simple_selector!(.wide));
    let dom: StyledElement = DOMElement::new("p", Some(attributes! {class=>wide}), vec![]).into();
    assert!(dom.does_rule_apply(&style));
}

// Taken from https://chromium.googlesource.com/chromium/blink/+/refs/heads/main/Source/core/css/html.css
static EXCLUDED: &[&str] = &[
    "head", "meta", "title", "link", "style", "script", "datalist", "param", "noframes", "template",
];

#[allow(unused_variables)]
static INHERITED: &[&str] = &[
    "azimuth",
    "border-collapse",
    "border-spacing",
    "caption-side",
    "color",
    "cursor",
    "direction",
    "elevation",
    "empty-cells",
    "font-family",
    "font-size",
    "font-style",
    "font-variant",
    "font-weight",
    "font",
    "letter-spacing",
    "line-height",
    "list-style-image",
    "list-style-position",
    "list-style-type",
    "list-style",
    "orphans",
    "pitch-range",
    "pitch",
    "quotes",
    "richness",
    "speak-header",
    "speak-numeral",
    "speak-punctuation",
    "speak",
    "speech-rate",
    "stress",
    "text-align",
    "text-indent",
    "text-transform",
    "visibility",
    "voice-family",
    "volume",
    "white-space",
    "widows",
    "word-spacing",
];

use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{char, none_of, space1},
    multi::separated_list1,
    sequence::{delimited, separated_pair},
    IResult,
};

use crate::html::dom::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

impl FromStr for DOMAttributes {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 || s.chars().all(char::is_whitespace) {
            return Ok(DOMAttributes::empty());
        }

        todo!()
    }
}

impl DOMNode {
    pub fn text(data: impl Into<String>) -> Self {
        Self {
            children: vec![],
            node_type: DOMNodeType::Text(data.into()),
        }
    }
    pub fn element(name: impl Display, attributes: DOMAttributes, children: Vec<DOMNode>) -> Self {
        Self {
            children,
            node_type: DOMNodeType::Element(DOMElement::new(name, Some(attributes))),
        }
    }
}

impl FromStr for DOMNode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('<') {
            return Ok(DOMNode::text(s));
        }
        let (node_open, s) = s.strip_prefix('<').ok_or(())?.split_once('>').ok_or(())?;
        let (s, node_close) = s.strip_suffix('>').ok_or(())?.rsplit_once("</").ok_or(())?;
        let open: DOMElement = node_open.parse()?;
        if open.tag_name.as_str() != node_close {
            return Err(());
        }
        if !open.tag_name.chars().all(char::is_alphanumeric)
            || !open.tag_name.chars().next().ok_or(())?.is_alphabetic()
        {
            return Err(());
        }
        Ok(DOMNode::element(
            open.tag_name,
            open.attributes,
            vec![s.parse()?],
        ))
    }
}

#[cfg(test)]
#[test]
fn test_node_parse() {
    let data = r#"<html><div class=nothing><h1>Hello, world</h1></div></html>"#;
    let target = DOMNode::element(
        "html",
        DOMAttributes::empty(),
        vec![DOMNode::element(
            "div",
            DOMAttributes(HashMap::from([(
                "class".to_string(),
                "nothing".to_string(),
            )])),
            vec![DOMNode::element(
                "h1",
                DOMAttributes::empty(),
                vec![DOMNode::text("Hello, world")],
            )],
        )],
    );
    assert_eq!(data.parse::<DOMNode>().unwrap(), target);
}

#[cfg(test)]
#[test]
fn test_parse_malformed() {
    let data = r#"<html></closing><opening></html>"#;
    assert!(data.parse::<DOMNode>().is_err());
    let data = r#"<123></123>"#;
    assert!(data.parse::<DOMNode>().is_err());
}

impl DOMElement {
    pub fn new(name: impl Display, attributes: Option<DOMAttributes>) -> Self {
        Self {
            tag_name: name.to_string(),
            attributes: attributes.unwrap_or(DOMAttributes(HashMap::new())),
        }
    }
}

impl FromStr for DOMElement {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((name, attrs)) = s.split_once(' ') {
            if let Ok((_, attributes)) = all_attr_parser(attrs) {
                // either make all attr values optional or default to an empty string
                let attributes: HashMap<String, String> = attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                Ok(DOMElement::new(name, Some(DOMAttributes(attributes))))
            } else {
                Err(())
            }
        } else {
            // Tag with no spaces
            Ok(DOMElement::new(s, None))
        }
    }
}

#[cfg(test)]
#[test]
fn test_tag_parse() {
    let data = r#"div"#;
    let target = DOMElement {
        tag_name: "div".to_string(),
        attributes: DOMAttributes(HashMap::new()),
    };
    assert_eq!(data.parse::<DOMElement>().unwrap(), target);

    let data = r#"div class=nothing"#;
    let target = DOMElement {
        tag_name: "div".to_string(),
        attributes: DOMAttributes(HashMap::from([(
            "class".to_string(),
            "nothing".to_string(),
        )])),
    };
    assert_eq!(data.parse::<DOMElement>().unwrap(), target);

    let data = r#"div attr1 attr2=two attr3='three' attr4="number four""#;
    let target = DOMElement {
        tag_name: "div".to_string(),
        attributes: DOMAttributes(HashMap::from([
            ("attr1".to_string(), "".to_string()),
            ("attr2".to_string(), "two".to_string()),
            ("attr3".to_string(), "three".to_string()),
            ("attr4".to_string(), "number four".to_string()),
        ])),
    };
    assert_eq!(data.parse::<DOMElement>().unwrap(), target);
}

// Attribute parsing below

fn parse_single_quoted(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of("\\\'"), '\\', tag("'"));
    let esc_or_empty = alt((esc, tag("")));
    let res = delimited(tag("'"), esc_or_empty, tag("'"))(input)?;
    Ok(res)
}

fn parse_double_quoted(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of("\\\""), '\\', tag("\""));
    let esc_or_empty = alt((esc, tag("")));
    let res = delimited(tag("\""), esc_or_empty, tag("\""))(input)?;
    Ok(res)
}

fn parse_unquoted(input: &str) -> IResult<&str, &str> {
    is_not(" \"'=<>`")(input)
}

fn value_parser(input: &str) -> IResult<&str, &str> {
    alt((parse_single_quoted, parse_double_quoted, parse_unquoted))(input)
}

fn name_parser(input: &str) -> IResult<&str, &str> {
    is_not(" \"'>/=")(input)
}

fn single_attr_parser(input: &str) -> IResult<&str, (&str, &str)> {
    let mut key_value = separated_pair(name_parser, char('='), value_parser);
    if let Ok((r, (k, v))) = key_value(input) {
        Ok((r, (k, v)))
    } else {
        let (r, res) = name_parser(input)?;
        Ok((r, (res, "")))
    }
}

fn all_attr_parser(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    separated_list1(space1, single_attr_parser)(input)
}

use nom::{
    branch::alt,
    multi::{many0, separated_list0},
    combinator::{opt, verify},
    character::complete::alphanumeric1,
    bytes::complete::{escaped, is_not, tag, take_until},
    character::complete::{char, none_of, space0, space1},
    sequence::{delimited, separated_pair, tuple},
    IResult,
};

use super::dom::*;
use std::collections::HashMap;
use std::fmt::Display;

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

/// Attempt to parse a string as a valid tag name
fn parse_tag_name(input: &str) -> IResult<&str, &str> {
    alphanumeric1(input)
}

/// Parse a tag in the form `</name>`, returning `name`
fn parse_close_tag(input: &str) -> IResult<&str, &str> {
    let (remaining, (_, name, _)) = tuple((tag("</"), parse_tag_name, char('>')))(input)?;
    Ok((remaining, name))
}

/// Parse a tag in the form `<name attr=value ...>`, returning the [`DOMElement`]
fn parse_open_tag(input: &str) -> IResult<&str, DOMElement> {
    // Parse input into the opening tag and the rest
    let parser = tuple((char('<'), space0, take_until(">"), char('>')));
    fn check(values: &(char, &str, &str, char)) -> bool {
        !values.2.contains("/")
    }
    let (rest, (_, _, tag, _)) = verify(parser, check)(input)?;
    // Parse out tag from name
    let (remaining, name) = parse_tag_name(tag)?;
    let attrs = if let Ok((_, (_, attrs))) = tuple((space1, all_attr_parser))(remaining) {
        attrs
    } else {
        vec![]
    };
    Ok((
        rest,
        DOMElement::new(
            name,
            Some(DOMAttributes(
                attrs
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            )),
        ),
    ))
}

/// Parse the content between an opening and closing tag, returning the text withing
fn parse_text(input: &str) -> IResult<&str, DOMNode> {
    let (remaining, res) = verify(take_until("<"), |s: &str| !s.starts_with("<") && s.len() > 0)(input)?;
    Ok((remaining, DOMNode::text(res)))
}

/// Parse the content between an opening and closing tab, returning the list of [`DOMNode`]'s within
fn parse_dom_node_contents(input: &str) -> IResult<&str, Vec<DOMNode>> {
    // let (remaining, node) = alt((parse_dom_node, parse_text))(input)?;
    // Ok((remaining, vec![node]))
    many0(alt((parse_dom_node, parse_text)))(input)
}

/// Parse a complete DOM tag, returning the [`DOMNode`]
pub fn parse_dom_node(input: &str) -> IResult<&str, DOMNode> {
    let parser = tuple((
        parse_open_tag,
        opt(parse_dom_node_contents),
        parse_close_tag,
    ));
    let (remaining, (open, contents, _)) =
        verify(parser, |(open, _, close)| &open.tag_name.as_str() == close)(input)?;
    Ok((
        remaining,
        DOMNode::element(open.tag_name, open.attributes, contents.unwrap_or(vec![])),
    ))
}

#[cfg(test)]
#[test]
fn test_node_parse() {
    let data = r#"<html><div class=nothing><h1></h1></div></html>"#;
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
                vec![],
            )],
        )],
    );

    assert_eq!(parse_dom_node(data).unwrap(), ("", target));

    let data = r#"<html><h1>Hello, world</h1></html>"#;
    let target = DOMNode::element(
    "html",
    DOMAttributes::empty(),
    vec![DOMNode::element(
            "h1",
            DOMAttributes::empty(),
            vec![DOMNode::text("Hello, world")],
        )],
    );

    assert_eq!(parse_dom_node(data).unwrap(), ("", target));
}

#[cfg(test)]
#[test]
fn test_parse_malformed() {
    let data = r#"<html></closing><opening></html>"#;

    assert!(parse_dom_node(data).is_err());
    let data = r#"<---></--->"#;
    assert!(parse_dom_node(data).is_err());
}

impl DOMElement {
    pub fn new(name: impl Display, attributes: Option<DOMAttributes>) -> Self {
        Self {
            tag_name: name.to_string(),
            attributes: attributes.unwrap_or(DOMAttributes(HashMap::new())),
        }
    }
}

#[cfg(test)]
#[test]
fn test_tag_parse() {
    let data = r#"<div>"#;
    let target = DOMElement {
        tag_name: "div".to_string(),
        attributes: DOMAttributes(HashMap::new()),
    };
    assert_eq!(parse_open_tag(data).unwrap(), ("", target));

    let data = r#"<div class=nothing>"#;
    let target = DOMElement {
        tag_name: "div".to_string(),
        attributes: DOMAttributes(HashMap::from([(
            "class".to_string(),
            "nothing".to_string(),
        )])),
    };
    assert_eq!(parse_open_tag(data).unwrap(), ("", target));

    let data = r#"<div attr1 attr2=two attr3='three' attr4="number four">"#;
    let target = DOMElement {
        tag_name: "div".to_string(),
        attributes: DOMAttributes(HashMap::from([
            ("attr1".to_string(), "".to_string()),
            ("attr2".to_string(), "two".to_string()),
            ("attr3".to_string(), "three".to_string()),
            ("attr4".to_string(), "number four".to_string()),
        ])),
    };
    assert_eq!(parse_open_tag(data).unwrap(), ("", target));
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
    separated_list0(space1, single_attr_parser)(input)
}

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until},
    character::complete::{alphanumeric1, anychar, char, multispace0, none_of, one_of, space1},
    combinator::{map, opt, value, verify},
    multi::{many0, many1, many_till},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use tracing::{span, Level};

use super::*;

static VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "command", "embed", "hr", "img", "input", "keygen", "link",
    "meta", "param", "source", "track", "wbr",
];
static RAW_TEXT_ELEMENTS: &[&str] = &["script", "style"];

#[derive(Debug, PartialEq, Clone)]
struct Tag<'a> {
    pub opening: bool,
    pub name: &'a str,
    pub attributes: DOMAttributes,
}

impl<'a> Tag<'a> {
    pub fn opening(name: &'a str, attributes: Option<impl Into<DOMAttributes>>) -> Self {
        let attributes = match attributes {
            Some(a) => a.into(),
            None => Default::default(),
        };
        Self {
            name,
            attributes,
            opening: true,
        }
    }
    pub fn closing(name: &'a str) -> Self {
        Self {
            name,
            attributes: Default::default(),
            opening: false,
        }
    }
}

// TODO: `Document` type holding doctype
pub fn document(input: &str) -> IResult<&str, DOMElement> {
    let span = span!(Level::DEBUG, "Parsing HTML");
    let _enter = span.enter();
    let (input, _) = ws(input)?;
    let (input, _) = opt(doctype)(input)?;
    let (input, _) = ws(input)?;
    let (input, root) = dom_element(input)?;
    let (input, _) = ws(input)?;
    Ok((input, root))
}

fn doctype(input: &str) -> IResult<&str, &str> {
    let (input, (_, _, t, _, _, _)) = tuple((
        tag_no_case("<!DOCTYPE"),
        many1(space),
        tag_no_case("html"),
        many0(space),
        opt(many0(none_of(">"))),
        char('>'),
    ))(input)?;
    Ok((input, t))
}

fn dom_element(input: &str) -> IResult<&str, DOMElement> {
    alt((
        void_element,
        raw_text_element,
        normal_element, /* rcdata_element, foreign_element*/
    ))(input)
}

fn void_element(input: &str) -> IResult<&str, DOMElement> {
    let (input, tag) = verify(alt((start_tag, start_void_tag)), |t| {
        VOID_ELEMENTS.contains(&t.name.to_lowercase().as_str())
    })(input)?;
    Ok((
        input,
        DOMElement::new(tag.name, Some(tag.attributes), vec![]),
    ))
}

#[cfg(test)]
#[test]
fn test_void() {
    let i = "<br>";
    let target = Ok((
        "",
        DOMElement {
            name: "br".to_string(),
            attributes: DOMAttributes(HashMap::new()),
            contents: vec![],
        },
    ));
    assert_eq!(void_element(i), target);
    let i = "<br/>";
    assert_eq!(void_element(i), target);
    let i = "<p/>";
    assert!(void_element(i).is_err());
}

fn normal_element(input: &str) -> IResult<&str, DOMElement> {
    if let Ok((input, (start, contents, _))) =
        verify(tuple((start_tag, normal_contents, end_tag)), |(s, _, e)| {
            s.name == e.name
        })(input)
    {
        return Ok((
            input,
            DOMElement::new(start.name, Some(start.attributes), contents),
        ));
    }
    // Most browsers will correct <b>...<b> -> <b>...</b>
    let (input, (start, contents, _)) = verify(
        tuple((start_tag, normal_contents, start_tag)),
        |(s, _, e)| s.name == e.name,
    )(input)?;
    Ok((
        input,
        DOMElement::new(start.name, Some(start.attributes), contents),
    ))
}

#[cfg(test)]
#[test]
fn test_normal_element() {
    let i = "<div>Test</div>";
    let target = DOMElement::new("div", None, vec!["Test".into()]);
    assert_eq!(normal_element(i), Ok(("", target)));

    let i = "<div><h1>Head</h1><h2>Head 2</h2></div>";
    let target = DOMElement::new(
        "div",
        None,
        vec![
            DOMElement::new("h1", None, vec!["Head".into()]).into(),
            DOMElement::new("h2", None, vec!["Head 2".into()]).into(),
        ],
    );
    assert_eq!(normal_element(i), Ok(("", target)));

    let i = "<b>Uh-oh<b>";
    let target = DOMElement::new("b", None, vec!["Uh-oh".into()]);
    assert_eq!(normal_element(i), Ok(("", target)));
}

// Text, character references, elements, comments
fn normal_contents(input: &str) -> IResult<&str, Vec<DOMContent>> {
    let content_dom_element = map(dom_element, |el| el.into());
    let possible = delimited(
        opt(comment),
        alt((content_dom_element, text_content)),
        opt(comment),
    );
    let (input, result) = many0(possible)(input)?;
    Ok((
        input,
        result
            .into_iter()
            .filter(|dc| {
                if let DOMContent::Text(s) = dc {
                    !s.is_empty()
                } else {
                    true
                }
            })
            .collect(),
    ))
}

fn text_content(input: &str) -> IResult<&str, DOMContent> {
    let (input, result): (&str, String) = map(
        verify(take_until("<"), |s: &str| !s.is_empty()),
        |el: &str| {
            // Truncate whitespace
            el.to_string()
                .split_whitespace()
                .intersperse(" ")
                .collect::<String>()
        },
    )(input)?;
    Ok((input, result.into()))
}

// <script> and <style>
fn raw_text_element(input: &str) -> IResult<&str, DOMElement> {
    let (input, start) = verify(start_tag, |t| {
        RAW_TEXT_ELEMENTS.contains(&t.name.to_lowercase().as_str())
    })(input)?;
    let (input, content) = map(many_till(anychar, named_end_tag(start.name)), |(v, _)| {
        v.into_iter().collect::<String>()
    })(input)?;
    Ok((
        input,
        DOMElement::new(start.name, Some(start.attributes), vec![content.into()]),
    ))
}

// Create a parser that searches  for a named closing tag
// Anonymous lifetime to capture 'name'
fn named_end_tag(name: &str) -> impl FnMut(&str) -> IResult<&str, &str> + '_ {
    move |input: &str| {
        delimited(
            tag("</"),
            terminated(tag_no_case(name), many0(space)),
            char('>'),
        )(input)
    }
}

#[cfg(test)]
#[test]
fn test_raw_text() {
    let i = r#"<script>let one = 2;</script>"#;
    let target = DOMElement::new("script", None, vec!["let one = 2;".into()]);
    assert_eq!(raw_text_element(i), Ok(("", target)));

    let i = r#"<script>let one = "</two>";</script>"#;
    let target = DOMElement::new("script", None, vec![r#"let one = "</two>";"#.into()]);
    assert_eq!(raw_text_element(i), Ok(("", target)));

    let i = r#"<style>html {}</script>"#;
    assert!(raw_text_element(i).is_err());
}

// An opening tag <elem>
fn start_tag(input: &str) -> IResult<&str, Tag> {
    let (input, (name, attributes, _)) = delimited(
        char('<'),
        tuple((tag_name, opt(attributes), many0(space))),
        char('>'),
    )(input)?;
    let attributes = attributes.unwrap_or_default();
    Ok((input, Tag::opening(name, Some(attributes))))
}

// An opening tag with terminator <elem/> (Only for void elements)
fn start_void_tag(input: &str) -> IResult<&str, Tag> {
    let (input, (name, attributes, _)) = delimited(
        char('<'),
        tuple((tag_name, opt(attributes), many0(space))),
        tag("/>"),
    )(input)?;
    let attributes = attributes.unwrap_or_default();
    Ok((input, Tag::opening(name, Some(attributes))))
}

// A closing tag </elem>
fn end_tag(input: &str) -> IResult<&str, Tag> {
    let (input, (name, _)) =
        delimited(tag("</"), tuple((tag_name, many0(space))), char('>'))(input)?;
    Ok((input, Tag::closing(name)))
}

#[cfg(test)]
#[test]
fn test_end_tag() {
    let target = Ok((
        "",
        Tag {
            name: "elem",
            opening: false,
            attributes: Default::default(),
        },
    ));
    let i = "</elem>";
    assert_eq!(end_tag(i), target);
    let i = "</elem   >";
    assert_eq!(end_tag(i), target);
    let i = "</elem disabled>";
    assert!(end_tag(i).is_err());
}

// The name of an HTML element
fn tag_name(input: &str) -> IResult<&str, &str> {
    alphanumeric1(input)
}

// A list of attributes
fn attributes(input: &str) -> IResult<&str, DOMAttributes> {
    let (input, attrs) = many0(preceded(space1, attribute))(input)?;
    let attrs: HashMap<String, String> = attrs.into_iter().collect();
    Ok((input, DOMAttributes(attrs)))
}

#[cfg(test)]
#[test]
fn test_attributes() {
    let i = r#" disabled attr=value attr2='value' attr3="multiple values""#;
    let attrs = DOMAttributes(HashMap::from([
        ("disabled".to_string(), "".to_string()),
        ("attr".to_string(), "value".to_string()),
        ("attr2".to_string(), "value".to_string()),
        ("attr3".to_string(), "multiple values".to_string()),
    ]));
    assert_eq!(attributes(i), Ok(("", attrs)));
}

// A single attribute
fn attribute(input: &str) -> IResult<&str, (String, String)> {
    let empty = map(attribute_name, |n| (n, "".to_string()));
    let unquoted = map(
        tuple((
            attribute_name,
            multispace0,
            char('='),
            multispace0,
            many1(none_of(" \t\r\n\0\"'>=")),
        )),
        |(name, .., value)| (name, value.into_iter().collect()),
    );
    let single_quoted = map(
        tuple((
            attribute_name,
            multispace0,
            char('='),
            multispace0,
            delimited(char('\''), take_until("'"), char('\'')),
        )),
        |(name, .., value)| (name, value.to_string()),
    );
    let double_quoted = map(
        tuple((
            attribute_name,
            multispace0,
            char('='),
            multispace0,
            delimited(char('"'), take_until("\""), char('"')),
        )),
        |(name, .., value)| (name, value.to_string()),
    );
    alt((single_quoted, double_quoted, unquoted, empty))(input)
}

fn attribute_name(input: &str) -> IResult<&str, String> {
    map(many1(none_of(" \t\r\n\0\"'>=")), |x| {
        x.into_iter().collect()
    })(input)
}

#[cfg(test)]
#[test]
fn test_attribute() {
    let i = r#"disabled"#;
    assert_eq!(
        attribute(i),
        Ok(("", ("disabled".to_string(), "".to_string())))
    );
    let i = r#"attr=value"#;
    assert_eq!(
        attribute(i),
        Ok(("", ("attr".to_string(), "value".to_string())))
    );
    let i = r#"attr='value'"#;
    assert_eq!(
        attribute(i),
        Ok(("", ("attr".to_string(), "value".to_string())))
    );
    let i = r#"attr="multiple words""#;
    assert_eq!(
        attribute(i),
        Ok(("", ("attr".to_string(), "multiple words".to_string())))
    );
}

fn comment(input: &str) -> IResult<&str, ()> {
    value((), delimited(tag("<!--"), take_until("-->"), tag("-->")))(input)
}

fn space(input: &str) -> IResult<&str, ()> {
    value((), one_of(" \t\r\n"))(input)
}

fn ws(input: &str) -> IResult<&str, ()> {
    alt((comment, value((), many0(space))))(input)
}

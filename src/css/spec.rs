use super::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take_until};
use nom::character::complete::{alphanumeric1, char, multispace0, multispace1, one_of};
use nom::combinator::{map, opt, value};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

///! Implements the CSS spec (https://github.com/antlr/grammars-v4/blob/master/css3/css3.g4)

pub fn stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    let (input, _) = ws(input)?;
    let (input, _) = many0(pair(charset, ws))(input)?;
    let (input, _) = many0(pair(import, ws))(input)?;
    let (input, rules) = many0(tuple((alt((ruleset /* TODO: media, page */,)), ws)))(input)?;
    let rules = rules.into_iter().map(|r| r.0).collect();
    Ok((input, Stylesheet { rules }))
}
#[cfg(test)]
#[test]
fn test_stylesheet() {
    let i = r#"/* W3.CSS 4.15 December 2020 by Jan Egil and Borge Refsnes */
@import "test.css";

html {
    box-sizing: border-box
}

*,*:before,*:after {
    box-sizing: inherit
}

/* Extract from normalize.css by Nicolas Gallagher and Jonathan Neal git.io/normalize */
html {
    -ms-text-size-adjust: 100%;
    -webkit-text-size-adjust: 100%
}

body {
    margin: 0
}
"#;
    let target = Stylesheet {
        rules: vec![
            Ruleset {
                selectors: vec![Selector::Simple(simple_selector!(html))],
                declarations: vec![],
            },
            Ruleset {
                selectors: vec![
                    Selector::Simple(simple_selector!(*)),
                    Selector::Compound(vec![simple_selector!(*), simple_selector!(:before)]),
                    Selector::Compound(vec![simple_selector!(*), simple_selector!(:after)]),
                ],
                declarations: vec![],
            },
            Ruleset {
                selectors: vec![Selector::Simple(simple_selector!(html))],
                declarations: vec![],
            },
            Ruleset {
                selectors: vec![Selector::Simple(simple_selector!(body))],
                declarations: vec![],
            },
        ],
    };
    assert_eq!(stylesheet(i), Ok(("", target)));
}

/// Parse an 'import' statement
fn import(input: &str) -> IResult<&str, String> {
    let (input, (_, _, url, _)) = tuple((
        tag_no_case("@import"),
        multispace0,
        alt((string, uri)),
        char(';'),
    ))(input)?;
    Ok((input, url))
}

#[cfg(test)]
#[test]
fn test_import() {
    let i = r#"@import "navigation.css";"#;
    let target = Ok(("", "navigation.css".to_string()));
    assert_eq!(import(i), target);
    let i = r#"@import url("navigation.css");"#;
    assert_eq!(import(i), target);
}

/// Parse a 'charset' statement
fn charset(input: &str) -> IResult<&str, String> {
    map(
        tuple((tag("@charset"), ws, string, ws, char(';'), ws)),
        |t| t.2,
    )(input)
}

/// Parse quoted string
fn string(input: &str) -> IResult<&str, String> {
    /// Parse double-quoted string
    fn string1(input: &str) -> IResult<&str, String> {
        let (input, (_, content, _)) = tuple((
            char('"'),
            many0(alt((is_not("\n\r\\\""), alt((tag("\\n"), tag("\\r")))))),
            char('"'),
        ))(input)?;
        let content = content.join("");
        Ok((input, content))
    }
    /// Parse single-quoted string
    fn string2(input: &str) -> IResult<&str, String> {
        let (input, (_, content, _)) = tuple((
            char('\''),
            many0(alt((is_not("\n\r\\'"), alt((tag("\\n"), tag("\\r")))))),
            char('\''),
        ))(input)?;
        let content = content.join("");
        Ok((input, content))
    }
    alt((string1, string2))(input)
}
#[cfg(test)]
#[test]
fn test_string() {
    let i = r#""Hello, world""#;
    let target = ("", "Hello, world".to_string());
    assert_eq!(string(i).unwrap(), target);

    let i = "'Hello, world'";
    let target = ("", "Hello, world".to_string());
    assert_eq!(string(i).unwrap(), target);

    let i = r#""Hello\nworld""#;
    let target = ("", "Hello\\nworld".to_string());
    assert_eq!(string(i).unwrap(), target);
}

/// Parse ruleset
fn ruleset(input: &str) -> IResult<&str, Ruleset> {
    let (input, (first, rest)) = tuple((
        selector,
        map(many0(tuple((char(','), multispace0, selector))), |v| {
            v.into_iter().map(|t| t.2).collect::<Vec<Selector>>()
        }),
    ))(input)?;
    let mut selectors = vec![first];
    selectors.extend(rest);
    let (input, _) = multispace0(input)?;
    let (input, _decl) = delimited(char('{'), take_until("}"), char('}'))(input)?;
    let (input, _) = multispace0(input)?;
    Ok((
        input,
        Ruleset {
            selectors,
            declarations: vec![],
        },
    ))
}

/// Parse selector
fn selector(input: &str) -> IResult<&str, Selector> {
    let (input, simple) = simple_selectors(input)?;
    if simple.len() == 1 {
        Ok((input, Selector::Simple(simple[0].clone())))
    } else {
        Ok((input, Selector::Compound(simple)))
    }
}

fn combinator(input: &str) -> IResult<&str, Combinator> {
    let (input, val) = one_of(" +>~")(input)?;
    let c = match val {
        ' ' => Combinator::Descendant,
        '>' => Combinator::Child,
        '+' => Combinator::NextSibling,
        '~' => Combinator::SubsequentSibling,
        _ => unreachable!(),
    };
    Ok((input, c))
}

fn simple_selectors(input: &str) -> IResult<&str, Vec<SimpleSelector>> {
    let hash = pair(tag("#"), name);
    let class = pair(tag("."), ident);
    let pseudo = pair(tag(":"), ident);
    let selectors = alt((hash, class /*attrib*/, pseudo));
    let element_or_universal = alt((ident, map(tag("*"), str::to_string)));
    let (input, (first, rest)) = tuple((element_or_universal, many0(selectors)))(input)?;
    let mut selectors = vec![first];
    selectors.extend(rest.iter().map(|(a, b)| format!("{}{}", a, b)));
    Ok((
        input,
        selectors.into_iter().map(|s| simple_selector(s)).collect(),
    ))
}

fn simple_selector(input: String) -> SimpleSelector {
    let mut it = input.chars();
    match it.next().unwrap() {
        '#' => SimpleSelector::ID(IDSelector(it.collect())),
        '.' => SimpleSelector::Class(ClassSelector(it.collect())),
        '*' => SimpleSelector::Universal,
        ':' => SimpleSelector::PseudoClass(PseudoClassSelector(it.collect())),
        '[' => todo!(),
        _ => SimpleSelector::Type(TypeSelector(input)),
    }
}

/// Parse CDO
fn cdo(input: &str) -> IResult<&str, ()> {
    value((), tag("<!--"))(input)
}
/// Parse CDC
fn cdc(input: &str) -> IResult<&str, ()> {
    value((), tag("-->"))(input)
}
/// Parse whitespace
fn s(input: &str) -> IResult<&str, &str> {
    alt((
        multispace1,
        delimited(tag("/*"), take_until("*/"), tag("*/")),
    ))(input)
}
/// Another whitespace parse
fn ws(input: &str) -> IResult<&str, ()> {
    value(
        (),
        many0(alt((
            delimited(tag("/*"), take_until("*/"), tag("*/")),
            multispace1,
        ))),
    )(input)
}
/// Parse URI
fn uri(input: &str) -> IResult<&str, String> {
    let (input, (_, url, _)) = delimited(
        tag("url("),
        tuple((multispace0, string, multispace0)),
        tag(")"),
    )(input)?;
    Ok((input, url))
}

/// Parse name
fn name(input: &str) -> IResult<&str, String> {
    let nmchar = alt((alphanumeric1, alt((tag("_"), tag("-")))));
    let (input, vals) = many1(nmchar)(input)?;
    Ok((input, vals.join("")))
}
/// Parse ident
fn ident(input: &str) -> IResult<&str, String> {
    let nmstart = alt((alphanumeric1, tag("_")));
    let nmchar = alt((alphanumeric1, alt((tag("_"), tag("-")))));
    let (input, (a, b, c)) = tuple((opt(tag("")), nmstart, many0(nmchar)))(input)?;
    let a = match a {
        Some(c) => c.to_string(),
        None => "".to_string(),
    };
    let identifier = format!("{}{}{}", a, b, c.into_iter().collect::<String>());
    Ok((input, identifier))
}
#[cfg(test)]
#[test]
fn test_name() {
    let i = "hello";
    let target = ("", "hello".to_string());
    assert_eq!(name(i).unwrap(), target);

    let i = "~hello";
    assert!(name(i).is_err());
}

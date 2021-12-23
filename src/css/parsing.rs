use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, take_while_m_n};
use nom::character::complete::{char, digit0, digit1, multispace0, one_of, space0};
use nom::combinator::{map_res, opt, verify};
use nom::multi::{many0, separated_list0, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair, tuple};
use nom::IResult;

use super::{Color, Declaration, Rule, Selector, SimpleSelector, Stylesheet, Unit, Value};

fn parse_stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    let (r, rules) = many0(tuple((parse_rule, multispace0)))(input)?;
    Ok((r, Stylesheet { rules: rules.into_iter().map(|(rule, _)| rule).collect() }))
}

#[cfg(test)]
#[test]
fn test_parse_stylesheet() {
    let i = r#"h1 {}
    h2 {}"#;
    let r1 = Rule {
        selectors: vec![Selector::Simple(SimpleSelector {
            tag_name: Some("h1".to_string()),
            ..Default::default()
        })],
        declarations: vec![],
    };
    let r2 = Rule {
        selectors: vec![Selector::Simple(SimpleSelector {
            tag_name: Some("h2".to_string()),
            ..Default::default()
        })],
        declarations: vec![],
    };
    let target = Stylesheet {rules: vec![r1, r2]};
    assert_eq!(parse_stylesheet(i).unwrap(), ("", target));
}

fn parse_rule(input: &str) -> IResult<&str, Rule> {
    let (r, (selectors, _)) = tuple((
        separated_list1(tuple((tag(","), space0)), parse_selector),
        space0,
    ))(input)?;
    let (r, _) = multispace0(r)?;
    // TODO: Allow trailing semicolon
    let (r, declarations) = delimited(
        char('{'),
        preceded(multispace0, separated_list0(tuple((tag(";"), multispace0)), parse_declaration)),
        tuple((multispace0, char('}'))),
    )(r)?;
    Ok((
        r,
        Rule {
            selectors,
            declarations,
        },
    ))
}

#[cfg(test)]
#[test]
fn test_parse_rule() {
    let i = r#"h1 {
    color: black;
    background-color: #112233
}"#;
    let target = Rule {
        selectors: vec![Selector::Simple(SimpleSelector {
            tag_name: Some("h1".to_string()),
            ..Default::default()
        })],
        declarations: vec![
            Declaration {
                name: "color".to_string(),
                value: Value::Keyword("black".to_string()),
            },
            Declaration {
                name: "background-color".to_string(),
                value: Value::ColorValue(Color {
                    r: 0x11,
                    g: 0x22,
                    b: 0x33,
                    a: 255,
                }),
            },
        ],
    };
    assert_eq!(parse_rule(i).unwrap(), ("", target));
}

fn parse_selector(input: &str) -> IResult<&str, Selector> {
    alt((parse_universal_selector, parse_simple_selector))(input)
}

fn parse_universal_selector(input: &str) -> IResult<&str, Selector> {
    let (res, _) = char('*')(input)?;
    Ok((res, Selector::Universal))
}

fn parse_simple_selector(input: &str) -> IResult<&str, Selector> {
    let (res, prefix) = opt(one_of("#."))(input)?;
    let (res, selector) = match prefix {
        Some('#') => {
            let (res, ident) = parse_identifier(res)?;
            (
                res,
                SimpleSelector {
                    id: Some(ident.to_string()),
                    ..Default::default()
                },
            )
        }
        Some('.') => {
            let (res, ident) = parse_identifier(res)?;
            (
                res,
                SimpleSelector {
                    class: Some(ident.to_string()),
                    ..Default::default()
                },
            )
        }
        None => {
            let (res, ident) = parse_identifier(res)?;
            (
                res,
                SimpleSelector {
                    tag_name: Some(ident.to_string()),
                    ..Default::default()
                },
            )
        }
        _ => unreachable!(),
    };
    Ok((res, Selector::Simple(selector)))
}

#[cfg(test)]
#[test]
fn test_parse_selector() {
    let i = "*";
    assert_eq!(parse_selector(i).unwrap(), ("", Selector::Universal));
    let i = ".class-name";
    assert_eq!(
        parse_selector(i).unwrap(),
        (
            "",
            Selector::Simple(SimpleSelector {
                class: Some("class-name".to_string()),
                ..Default::default()
            })
        )
    );
    let i = "#id-name";
    assert_eq!(
        parse_selector(i).unwrap(),
        (
            "",
            Selector::Simple(SimpleSelector {
                id: Some("id-name".to_string()),
                ..Default::default()
            })
        )
    );
    let i = "tag-name";
    assert_eq!(
        parse_selector(i).unwrap(),
        (
            "",
            Selector::Simple(SimpleSelector {
                tag_name: Some("tag-name".to_string()),
                ..Default::default()
            })
        )
    );
}

fn parse_declaration(input: &str) -> IResult<&str, Declaration> {
    let (res, (key, value)) = separated_pair(
        parse_identifier,
        tuple((space0, char(':'), space0)),
        parse_value,
    )(input)?;
    Ok((
        res,
        Declaration {
            name: key.to_string(),
            value,
        },
    ))
}

#[cfg(test)]
#[test]
fn test_parse_declaration() {
    let i = "font-size: 12px";
    assert_eq!(
        parse_declaration(i).unwrap(),
        (
            "",
            Declaration {
                name: "font-size".to_string(),
                value: Value::Measurement(12.0, Unit::Px)
            }
        )
    );
}

fn parse_value(input: &str) -> IResult<&str, Value> {
    if let Ok((res, color)) = parse_color(input) {
        return Ok((res, Value::ColorValue(color)));
    }
    if let Ok((res, (value, unit))) = tuple((parse_number, parse_unit))(input) {
        return Ok((res, Value::Measurement(value, unit)));
    }
    let (res, ident) = parse_identifier(input)?;
    Ok((res, Value::Keyword(ident.to_string())))
}

/// '12' -> `12.0`
fn parse_integer_to_float(input: &str) -> IResult<&str, f32> {
    let (res, num) = digit1(input)?;
    Ok((res, num.parse().unwrap()))
}

/// '.5' -> `0.5`
/// '0.5' -> `0.5`
fn parse_float(input: &str) -> IResult<&str, f32> {
    let (res, num) = tuple((digit0, char('.'), digit1))(input)?;
    Ok((res, format!("{}.{}", num.0, num.2).parse().unwrap()))
}

fn parse_number(input: &str) -> IResult<&str, f32> {
    alt((parse_float, parse_integer_to_float))(input)
}

#[cfg(test)]
#[test]
fn test_parse_value() {
    let i = "#00000000";
    assert_eq!(
        parse_value(i).unwrap(),
        (
            "",
            Value::ColorValue(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            })
        )
    );
    let i = "12.5px";
    assert_eq!(
        parse_value(i).unwrap(),
        ("", Value::Measurement(12.5, Unit::Px))
    );
    let i = "a_name";
    assert_eq!(
        parse_value(i).unwrap(),
        ("", Value::Keyword("a_name".to_string()))
    );
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

fn parse_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;
    if let Ok((input, (r, g, b, a))) =
        tuple((hex_primary, hex_primary, hex_primary, hex_primary))(input)
    {
        return Ok((input, Color { r, g, b, a }));
    }
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;
    Ok((
        input,
        Color {
            r: red,
            g: green,
            b: blue,
            a: 255,
        },
    ))
}

#[cfg(test)]
#[test]
fn test_parse_color() {
    let i = "#ffffff";
    let target = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    assert_eq!(parse_color(i).unwrap(), ("", target));

    let i = "#000000ff";
    let target = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    assert_eq!(parse_color(i).unwrap(), ("", target));
}

fn parse_unit(input: &str) -> IResult<&str, Unit> {
    let (res, unit) = alt((tag("px"),))(input)?;
    Ok((
        res,
        match unit {
            "px" => Unit::Px,
            _ => unimplemented!(),
        },
    ))
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    verify(is_not(" \t\r\n;:"), |s: &str| !s.starts_with("--"))(input)
}

#[cfg(test)]
#[test]
fn test_parse_ident() {
    let i = "test";
    assert!(parse_identifier(i).is_ok());
    let i = "_test";
    assert!(parse_identifier(i).is_ok());
    let i = "--test";
    assert!(parse_identifier(i).is_err());
}

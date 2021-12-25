use super::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take_until, take};
use nom::character::complete::{alphanumeric1, char, digit1, multispace0, multispace1, one_of};
use nom::combinator::{map, opt, value, verify};
use nom::multi::{many0, many1, many_m_n};
use nom::sequence::{delimited, pair, terminated, tuple};
use nom::{AsChar, IResult};

///! Implements the CSS spec (https://github.com/antlr/grammars-v4/blob/master/css3/css3.g4)

pub fn stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    let (input, _) = ws(input)?;
    let (input, _) = many0(pair(charset, ws))(input)?;
    let (input, _) = many0(pair(import, ws))(input)?;
    let (input, rules) = many0(ruleset)(input)?;
    let (input, _) = ws(input)?;
    // let rules = rules.into_iter().map(|r| r.0).collect();
    Ok((input, Stylesheet { rules }))
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
    let (input, selectors) = selector_group(input)?;
    let (input, _) = ws(input)?;
    let (input, declarations) = delimited(pair(char('{'), ws), declaration_list, pair(char('}'), ws))(input)?;
    let (input, _) = ws(input)?;
    Ok((
        input,
        Ruleset {
            selectors,
            declarations,
        },
    ))
}
#[cfg(test)]
#[test]
fn test_ruleset() {
    let i = r#"html {
    box-sizing: border-box
}"#;
    let target = Ruleset {
        selectors: vec![Selector::Simple(simple_selector!(html))],
        declarations: vec![Declaration::new("box-sizing", Value::textual(TextValue::keyword("border-box")))],
    };
    assert_eq!(ruleset(i), Ok(("", target)));
}

/// Parse comma seperated groups of selectors
fn selector_group(input: &str) -> IResult<&str, Vec<Selector>> {
    let (input, (first, rest)) = pair(
        selector,
        map(many0(tuple((char(','), opt(ws), selector))), |v| {
            v.into_iter().map(|t| t.2).collect::<Vec<Selector>>()
        }),
    )(input)?;
    Ok((input, [first].into_iter().chain(rest).collect()))
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

/// Parse list of declarations
fn declaration_list(input: &str) -> IResult<&str, Vec<Declaration>> {
    let (input, (_, first, _, rest)) = tuple((
        many0(pair(char(';'), ws)),
        declaration,
        ws,
        many0(map(tuple((char(';'), ws, opt(declaration))), |t| t.2)),
    ))(input)?;
    Ok((input, [first].into_iter().chain(rest.into_iter().filter_map(|v| v)).collect()))
}

/// Parse single declaration
fn declaration(input: &str) -> IResult<&str, Declaration> {
    let (input, (prop, _, _, value, _)) =
        tuple((property, char(':'), ws, expr, opt(priority)))(input)?;
    Ok((input, Declaration { name: prop, value }))
}

/// Parse property
fn property(input: &str) -> IResult<&str, String> {
    alt((ident, variable))(input)
}

/// Parse expression
fn expr(input: &str) -> IResult<&str, Value> {
    let (input, (result, others)) = pair(term, many0(pair(opt(operator), term)))(input)?;
    if others.len() > 0 {
        let all = [(None, result)].into_iter().chain(others).collect();
        Ok((input, Value::Multiple(MultiValue(all))))
    } else {
        Ok((input, result))
    }
}

/// Parse a term
fn term(input: &str) -> IResult<&str, Value> {
    let (input, val) = alt((
        function,
        terminated(percentage, ws),
        terminated(dimension, ws),
        terminated(number, ws),
        terminated(map(string, |s| Value::Textual(TextValue::String(s))), ws),
        terminated(map(ident, |s| Value::Textual(TextValue::Keyword(s))), ws),
        terminated(map(variable, |s| Value::Textual(TextValue::Keyword(s))), ws),
        terminated(map(uri, |s| Value::Textual(TextValue::Url(s))), ws),
        terminated(hexcolor, ws),
        // calc,
    ))(input)?;
    // Apply transformations
    match val.clone() {
        Value::Textual(TextValue::Keyword(s)) => Ok((input, keyword_to_value(s).unwrap_or(val))),
        Value::Function(f) => Ok((input, function_to_value(f).unwrap_or(val))),
        _ => Ok((input, val))
    }
}

fn number(input: &str) -> IResult<&str, Value> {
    let (input, (sign, number)) = pair(opt(one_of("+-")), digit1)(input)?;
    let sign = sign.unwrap_or('+');
    let val = format!("{}{}", sign, number).parse().unwrap();
    Ok((input, Value::Numeric(NumericValue::Number(val))))
}
fn percentage(input: &str) -> IResult<&str, Value> {
    let (input, (sign, number, _)) = tuple((opt(one_of("+-")), digit1, char('%')))(input)?;
    let sign = sign.unwrap_or('+');
    let val = format!("{}{}", sign, number).parse().unwrap();
    Ok((input, Value::Numeric(NumericValue::Percentage(val))))
}
fn dimension(input: &str) -> IResult<&str, Value> {
    let (input, (value, unit)) = pair(number, dimension_unit)(input)?;
    if let Value::Numeric(v) = value {
        Ok((input, Value::Dimension(v, unit)))
    } else {
        unreachable!()
    }
}
fn dimension_unit(input: &str) -> IResult<&str, Unit> {
    let abs = alt((
        tag("px"),
        tag("cm"),
        tag("mm"),
        tag("in"),
        tag("pt"),
        tag("pc"),
        tag("q"),
    ));
    let font_rel = alt((tag("em"), tag("ex"), tag("ch"), tag("rem")));
    let vp_rel = alt((tag("vw"), tag("vh"), tag("vmin"), tag("vmax")));
    let length = alt((abs, font_rel, vp_rel));
    let (input, unit): (&str, &str) = alt((
        length,
        tag("ms"),
        tag("s"),
        // Freq
        tag("hz"),
        tag("khz"),
        // Resolution
        tag("dpi"),
        tag("dpcm"),
        tag("dppx"),
        // Angle
        tag("deg"),
        tag("rad"),
        tag("grad"),
        tag("turn"),
    ))(input)?;
    let unit = match unit {
        "px" => Unit::Px,
        _ => todo!()
    };
    Ok((input, unit))
}
fn hexcolor(input: &str) -> IResult<&str, Value> {
    let is_hex_str = |c: &str| c.bytes().all(|c| c.is_hex_digit());
    let hex_val = verify(take::<usize, &str, _>(2), is_hex_str);
    let (input, (_, values)) = pair(char('#'), many_m_n(3, 4, hex_val))(input)?;
    let values: Vec<u8> = values.into_iter().map(|v| u8::from_str_radix(v, 16).expect("Passed an invalid hex value")).collect();
    let values: [u8; 4] = match &values.len() {
        4 => values.try_into().unwrap(),
        3 => values.into_iter().chain([255u8]).collect::<Vec<u8>>().try_into().unwrap(),
        _ => unreachable!()
    };
    let col = ColorValue::new(values.as_slice());
    Ok((input, Value::Color(col)))
}
fn calc(_input: &str) -> IResult<&str, Value> {
    todo!()
}
fn function(input: &str) -> IResult<&str, Value> {
    let (input, (name, _, _, args, _, _)) = tuple((ident, char('('), ws, expr, char(')'), ws))(input)?;
    if let Value::Multiple(v) = args {
        let value = v.0;
        if value.len() == 1 {
            // Just a single value
            Ok((input, Value::Function(FunctionValue(name, value.into_iter().map(|v| v.1).collect()))))
        } else {
            if value[1..].iter().all(|t| if let Some(Operator::Comma) = t.0 {true} else {false}) {
                // A comma-separated argument list
                Ok((input, Value::Function(FunctionValue(name, value.into_iter().map(|v| v.1).collect()))))
            } else {
                // Not comma seperated? Bail lol
                unimplemented!()
            }
        }
    } else {
        Ok((input, Value::Function(FunctionValue(name, vec![args]))))
    }
}

fn operator(input: &str) -> IResult<&str, Operator> {
    let (input, (op, _)) = pair(alt((char('/'), char(','), char(' '), char('='))), ws)(input)?;
    let op = match op {
        '/' => Operator::Slash,
        ',' => Operator::Comma,
        ' ' => Operator::Space,
        '=' => Operator::Equals,
        _ => unreachable!()
    };
    Ok((input, op))
}
/// Parse priority
fn priority(input: &str) -> IResult<&str, ()> {
    value((), pair(tag("important"), ws))(input)
}

#[cfg(test)]
#[test]
fn test_declaration_list() {
    let i = r#"color: black;
background-color: rgb(197,93,161)"#;
    let target = vec![
        Declaration {
            name: "color".to_string(),
            value: Value::Color(BLACK),
        },
        Declaration {
            name: "background-color".to_string(),
            value: Value::Color(ColorValue {
                r: 197,
                g: 93,
                b: 161,
                a: 255,
            }),
        },
    ];
    assert_eq!(declaration_list(i), Ok(((""), target)))
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
    let (input, (pref, start, rest)) = tuple((opt(char('-')), nmstart, many0(nmchar)))(input)?;
    let pref = match pref {
        Some(c) => c.to_string(),
        None => "".to_string(),
    };
    let identifier = format!("{}{}{}", pref, start, rest.into_iter().collect::<String>());
    Ok((input, identifier))
}
/// Parse variable
fn variable(input: &str) -> IResult<&str, String> {
    let nmstart = alt((alphanumeric1, tag("_")));
    let nmchar = alt((alphanumeric1, alt((tag("_"), tag("-")))));
    let (input, (a, b, c)) = tuple((tag("--"), nmstart, many0(nmchar)))(input)?;
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

use super::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take, take_until};
use nom::character::complete::{alphanumeric1, char, digit1, multispace0, multispace1, one_of};
use nom::combinator::{map, opt, peek, value, verify};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::{AsChar, IResult};
use tracing::{span, Level};

///! Implements the CSS spec (https://github.com/antlr/grammars-v4/blob/master/css3/css3.g4)

pub fn stylesheet(input: &str) -> IResult<&str, Stylesheet> {
    let span = span!(Level::DEBUG, "Parsing Stylesheet");
    let _enter = span.enter();
    let (input, _) = ws(input)?;
    let (input, _) = many0(pair(charset, ws))(input)?;
    let (input, _) = many0(pair(import, ws))(input)?;
    let (input, rules) = many0(ruleset)(input)?;
    // Skip over any rules with empty selectors/bodies - these are either useless or invalid,
    // and the parser has returned an empty rule
    let rules = rules
        .into_iter()
        .filter(|r| !r.selectors.is_empty() && !r.declarations.is_empty())
        .collect();
    let (input, _) = ws(input)?;
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
    let (input, body) =
        delimited(pair(char('{'), ws), take_until("}"), pair(char('}'), ws))(input)?;
    let (input, _) = ws(input)?;
    if let Ok((_, declarations)) = declaration_list(body) {
        Ok((
            input,
            Ruleset {
                selectors,
                declarations,
            },
        ))
    } else {
        // Wasn't able to parse the body of this ruleset - return an 'empty' rule and move on
        Ok((
            input,
            Ruleset {
                selectors: vec![],
                declarations: vec![],
            },
        ))
    }
}
#[cfg(test)]
#[test]
fn test_ruleset() {
    let i = r#"html {
    box-sizing: border-box
}"#;
    let target = Ruleset {
        selectors: vec![Selector::Simple(simple_selector!(html))],
        declarations: vec![Declaration::new(
            "box-sizing",
            Value::Keyword("border-box".to_string()),
        )],
    };
    assert_eq!(ruleset(i), Ok(("", target)));
}

/*
a selector group is made up of selectors split by commas
a selector is a simpleSelectorSequence split by combinators (+,>,~, )
a simpleSelectorSequence is a simple selector optionally followed by more simple selectors - i.e html, html.name, #a#b.c
simpleSelectorSequence selectors apply when *all* their constituent selectors apply
selectors split by combinators are more complex and represent relationships ('h1 h2' applies when h2 is inside of h1, 'div > p' applies when a div is a direct child of a p)
*/
/// Parse comma seperated groups of selectors
fn selector_group(input: &str) -> IResult<&str, Vec<Selector>> {
    let (input, (first, rest)) = pair(
        selector,
        map(many0(tuple((char(','), ws, selector))), |v| {
            v.into_iter().map(|t| t.2).collect::<Vec<Selector>>()
        }),
    )(input)?;
    Ok((input, [first].into_iter().chain(rest).collect()))
}

/// Parse selector
fn selector(input: &str) -> IResult<&str, Selector> {
    alt((combinator_selector, selector_sequence))(input)
}

fn selector_sequence(input: &str) -> IResult<&str, Selector> {
    let (input, seq) = terminated(simple_selector_sequence, ws)(input)?;
    if seq.len() == 1 {
        Ok((input, Selector::Simple(seq.into_iter().next().unwrap())))
    } else {
        Ok((input, Selector::Compound(seq)))
    }
}

fn combinator_selector(input: &str) -> IResult<&str, Selector> {
    let (input, (first, combinated)) = pair(
        simple_selector_sequence,
        many1(map(
            tuple((combinator, simple_selector_sequence, ws)),
            |(com, sels, _)| (com, sels),
        )),
    )(input)?;

    // Wrap `combinated` into temporary type so we can make it into a flat array
    enum Temp {
        Selector(Vec<SimpleSelector>),
        Combinator(Combinator),
    }
    let arr: Vec<Temp> = [Temp::Selector(first)]
        .into_iter()
        .chain(
            combinated
                .into_iter()
                .flat_map(|(com, sel)| [Temp::Combinator(com), Temp::Selector(sel)]),
        )
        .collect();
    fn peeler(v: &[Temp]) -> Selector {
        if v.len() == 3 {
            match v {
                [Temp::Selector(s1), Temp::Combinator(c), Temp::Selector(s2)] => {
                    combinator_selector!(
                        Selector::Compound(s1.clone()),
                        *c,
                        Selector::Compound(s2.clone())
                    )
                }
                _ => unreachable!(),
            }
        } else {
            let (start, rest) = v.split_at(2);
            match start {
                [Temp::Selector(s1), Temp::Combinator(c)] => {
                    combinator_selector!(Selector::Compound(s1.clone()), *c, peeler(rest))
                }
                _ => unreachable!(),
            }
        }
    }
    Ok((input, peeler(arr.as_slice())))
}

#[cfg(test)]
#[test]
fn test_combinator_selectors() {
    let i = "div > p";
    let target = combinator_selector!(
        compound_selector![simple_selector!(div)],
        Combinator::Child,
        compound_selector![simple_selector!(p)]
    );
    assert_eq!(selector(i), Ok(("", target)));
    let i = "div + p";
    let target = combinator_selector!(
        compound_selector![simple_selector!(div)],
        Combinator::NextSibling,
        compound_selector![simple_selector!(p)]
    );
    assert_eq!(selector(i), Ok(("", target)));

    let i = "div ~ p";
    let target = combinator_selector!(
        compound_selector![simple_selector!(div)],
        Combinator::SubsequentSibling,
        compound_selector![simple_selector!(p)]
    );
    assert_eq!(selector(i), Ok(("", target)));

    let i = "div p";
    let target = combinator_selector!(
        compound_selector![simple_selector!(div)],
        Combinator::Descendant,
        compound_selector![simple_selector!(p)]
    );
    assert_eq!(selector(i), Ok(("", target)));

    let i = "a b > c";
    let target = combinator_selector!(
        compound_selector![simple_selector!(a)],
        Combinator::Descendant,
        combinator_selector!(
            compound_selector![simple_selector!(b)],
            Combinator::Child,
            compound_selector![simple_selector!(c)]
        )
    );
    assert_eq!(selector(i), Ok(("", target)));
}

fn combinator(input: &str) -> IResult<&str, Combinator> {
    let (input, val) = alt((
        terminated(value(' ', ws), peek(simple_selector_sequence)),
        delimited(opt(ws), one_of("+>~"), opt(ws)),
    ))(input)?;
    let c = match val {
        ' ' => Combinator::Descendant,
        '>' => Combinator::Child,
        '+' => Combinator::NextSibling,
        '~' => Combinator::SubsequentSibling,
        _ => unreachable!(),
    };
    Ok((input, c))
}

fn simple_selector_sequence(input: &str) -> IResult<&str, Vec<SimpleSelector>> {
    let hash = map(pair(tag("#"), name), |(a, b)| format!("{}{}", a, b));
    let class = map(pair(tag("."), ident), |(a, b)| format!("{}{}", a, b));
    let pseudo = map(pair(tag(":"), ident), |(a, b)| format!("{}{}", a, b));
    let selectors = alt((hash, class /*attrib*/, pseudo));

    // Hack: Can't reuse FnMut so we just create it twice
    let hash = map(pair(tag("#"), name), |(a, b)| format!("{}{}", a, b));
    let class = map(pair(tag("."), ident), |(a, b)| format!("{}{}", a, b));
    let pseudo = map(pair(tag(":"), ident), |(a, b)| format!("{}{}", a, b));
    let selectors2 = alt((hash, class /*attrib*/, pseudo));

    let element_or_universal = alt((ident, map(tag("*"), str::to_string)));
    let (input, (first, rest)) =
        tuple((alt((element_or_universal, selectors)), many0(selectors2)))(input)?;
    let mut selectors = vec![first];
    selectors.extend(rest);
    Ok((input, selectors.into_iter().map(simple_selector).collect()))
}

fn simple_selector(input: String) -> SimpleSelector {
    let mut it = input.chars();
    #[allow(unreachable_code)]
    match it.next().unwrap() {
        '#' => SimpleSelector::ID(it.collect()),
        '.' => SimpleSelector::Class(it.collect()),
        '*' => SimpleSelector::Universal,
        ':' => SimpleSelector::PseudoClass(it.collect()),
        '[' => SimpleSelector::Attribute(todo!()),
        _ => SimpleSelector::Type(input),
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
    Ok((
        input,
        [first]
            .into_iter()
            .chain(rest.into_iter().flatten())
            .collect(),
    ))
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
    let (input, (result, others)) = pair(term, many0(pair(operator, term)))(input)?;
    if !others.is_empty() {
        let all = [(None, result)]
            .into_iter()
            .chain(others.into_iter().map(|(op, t)| (Some(op), t)))
            .collect();
        Ok((input, Value::Multiple(MultiValue(all))))
    } else {
        Ok((input, result))
    }
}

#[cfg(test)]
#[test]
fn test_expr() {
    use crate::css::Value::{Keyword, Length};
    let i = "5em auto";
    let target = Ok((
        "",
        Value::Multiple(MultiValue(vec![
            (None, Length(5.0, Unit::Em)),
            (Some(Operator::Space), Keyword("auto".to_string())),
        ])),
    ));
    assert_eq!(expr(i), target);
}

/// Parse a term
fn term(input: &str) -> IResult<&str, Value> {
    let (input, val) = alt((
        function,
        percentage,
        dimension,
        number,
        map(string, Value::String),
        map(ident, Value::Keyword),
        map(variable, Value::Keyword),
        map(uri, Value::Url),
        hexcolor,
        // calc,
    ))(input)?;
    // Apply transformations
    match val.clone() {
        Value::Keyword(s) => Ok((input, keyword_to_value(s).unwrap_or(val))),
        Value::Function(f) => Ok((input, function_to_value(f).unwrap_or(val))),
        _ => Ok((input, val)),
    }
}

fn number(input: &str) -> IResult<&str, Value> {
    let (input, (sign, number)) = pair(opt(one_of("+-")), digit1)(input)?;
    let sign = sign.unwrap_or('+');
    let val = format!("{}{}", sign, number).parse().unwrap();
    Ok((input, Value::Number(val)))
}
fn percentage(input: &str) -> IResult<&str, Value> {
    let (input, (sign, number, _)) = tuple((opt(one_of("+-")), digit1, char('%')))(input)?;
    let sign = sign.unwrap_or('+');
    let val = format!("{}{}", sign, number).parse().unwrap();
    Ok((input, Value::Percentage(val)))
}
fn dimension(input: &str) -> IResult<&str, Value> {
    let (input, (value, unit)) = pair(number, dimension_unit)(input)?;
    if let Value::Number(v) = value {
        Ok((input, Value::Length(v, unit)))
    } else {
        unreachable!()
    }
}
fn dimension_unit(input: &str) -> IResult<&str, Unit> {
    let abs = alt((
        tag_no_case("px"),
        tag_no_case("cm"),
        tag_no_case("mm"),
        tag_no_case("in"),
        tag_no_case("pt"),
        tag_no_case("pc"),
        tag_no_case("q"),
    ));
    let font_rel = alt((
        tag_no_case("em"),
        tag_no_case("ex"),
        tag_no_case("ch"),
        tag_no_case("rem"),
    ));
    let vp_rel = alt((
        tag_no_case("vw"),
        tag_no_case("vh"),
        tag_no_case("vmin"),
        tag_no_case("vmax"),
    ));
    let length = alt((abs, font_rel, vp_rel));
    let (input, unit): (&str, &str) = alt((
        length,
        tag_no_case("ms"),
        tag_no_case("s"),
        // Freq
        tag_no_case("hz"),
        tag_no_case("khz"),
        // Resolution
        tag_no_case("dpi"),
        tag_no_case("dpcm"),
        tag_no_case("dppx"),
        // Angle
        tag_no_case("deg"),
        tag_no_case("rad"),
        tag_no_case("grad"),
        tag_no_case("turn"),
    ))(input)?;
    let unit = match unit.to_lowercase().as_str() {
        "px" => Unit::Px,
        "cm" => Unit::Cm,
        "mm" => Unit::Mm,
        "em" => Unit::Em,
        _ => todo!("{}", unit),
    };
    Ok((input, unit))
}
fn hexcolor(input: &str) -> IResult<&str, Value> {
    let is_hex_str = |c: &str| c.bytes().all(|c| c.is_hex_digit());
    let long_form = map(verify(take::<usize, &str, _>(6), is_hex_str), |s| {
        s.to_string()
    });
    let short_form = verify(
        map(take::<usize, &str, _>(3), |val| {
            val.chars()
                .flat_map(|c| [c, c].into_iter())
                .collect::<String>()
        }),
        is_hex_str,
    );
    let (input, hex_val) = preceded(char('#'), alt((long_form, short_form)))(input)?;
    // We should be provided a string of length 3 or 6, with 3 being promoted to 6
    assert_eq!(hex_val.len(), 6);
    let hex_val: Vec<char> = hex_val.chars().collect();
    let mut values: Vec<u8> = hex_val
        .as_slice()
        .chunks(2)
        .map(|v| u8::from_str_radix(v.iter().collect::<String>().as_str(), 16).unwrap())
        .collect();
    values.push(255);

    let col = ColorValue::new(values.as_slice());
    Ok((input, Value::Color(col)))
}
#[cfg(test)]
#[test]
fn test_hexcolor() {
    let i = "#112233";
    let target = Value::Color(ColorValue::new(&[0x11, 0x22, 0x33, 0xff]));
    assert_eq!(hexcolor(i), Ok(("", target)));

    let i = "#123";
    let target = Value::Color(ColorValue::new(&[0x11, 0x22, 0x33, 0xff]));
    assert_eq!(hexcolor(i), Ok(("", target)));

    let i = "#ggg";
    assert!(hexcolor(i).is_err());
}

fn _calc(_input: &str) -> IResult<&str, Value> {
    todo!()
}
fn function(input: &str) -> IResult<&str, Value> {
    let (input, (name, _, _, args, _, _)) =
        tuple((ident, char('('), ws, expr, char(')'), ws))(input)?;
    if let Value::Multiple(v) = args {
        let value = v.0;
        if value.len() == 1 {
            // Just a single value
            Ok((
                input,
                Value::Function(FunctionValue(
                    name,
                    value.into_iter().map(|v| v.1).collect(),
                )),
            ))
        } else if value[1..]
            .iter()
            .all(|t| matches!(t.0, Some(Operator::Comma)))
        {
            // A comma-separated argument list
            Ok((
                input,
                Value::Function(FunctionValue(
                    name,
                    value.into_iter().map(|v| v.1).collect(),
                )),
            ))
        } else {
            // Not comma separated? Bail lol
            unimplemented!()
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
        _ => unreachable!(),
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

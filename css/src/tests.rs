use super::parsing::stylesheet;
use super::*;

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
                declarations: vec![Declaration::new("box-sizing", Value::Keyword("border-box"))],
            },
            Ruleset {
                selectors: vec![
                    Selector::Simple(simple_selector!(*)),
                    Selector::Compound(vec![simple_selector!(*), simple_selector!(:before)]),
                    Selector::Compound(vec![simple_selector!(*), simple_selector!(:after)]),
                ],
                declarations: vec![Declaration::new("box-sizing", Value::Keyword("inherit"))],
            },
            Ruleset {
                selectors: vec![Selector::Simple(simple_selector!(html))],
                declarations: vec![
                    Declaration::new("-ms-text-size-adjust", Value::Percentage(100.0)),
                    Declaration::new("-webkit-text-size-adjust", Value::Percentage(100.0)),
                ],
            },
            Ruleset {
                selectors: vec![Selector::Simple(simple_selector!(body))],
                declarations: vec![Declaration::new("margin", Value::Number(0.0))],
            },
        ],
    };
    assert_eq!(stylesheet(i), Ok(("", target)));
}

#[cfg(test)]
#[test]
fn test_invalid_rule() {
    let i = r#"h1 {
    BLAHBALH
}
h2 {
    color: black;
}"#;
    let target = Stylesheet {
        rules: vec![Ruleset {
            selectors: vec![Selector::Simple(simple_selector!(h2))],
            declarations: vec![Declaration::new("color", Value::Color(keywords::BLACK))],
        }],
    };
    assert_eq!(stylesheet(i), Ok(("", target)))
}

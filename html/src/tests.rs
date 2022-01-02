use super::*;

#[test]
fn test_document() {
    let i = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8"/>
        <title>The minimal, valid HTML5 document</title>
    </head>
    <body>
        <!-- User-visible content goes in the body -->
        <p>Some paragraph</p>
        Some untagged text
    </body>
</html>"#;
    let target = DOMElement::new(
        "html",
        Some(attributes!(lang=>en)),
        vec![
            DOMElement::new(
                "head",
                None,
                vec![
                    DOMElement::new("meta", Some(attributes!(charset=>utf-8)), vec![]).into(),
                    DOMElement::new(
                        "title",
                        None,
                        vec!["The minimal, valid HTML5 document".into()],
                    )
                    .into(),
                ],
            )
            .into(),
            DOMElement::new(
                "body",
                None,
                vec![
                    DOMElement::new("p", None, vec!["Some paragraph".into()]).into(),
                    "Some untagged text".into(),
                ],
            )
            .into(),
        ],
    );
    assert_eq!(document(i), Ok(("", target)));
}

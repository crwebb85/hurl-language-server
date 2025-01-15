use super::primitives::escaped_unicode_parser;
use super::types::{InterpolatedString, InterpolatedStringPart, KeyValue};
use super::{primitives::sp_parser, template::template_parser};
use chumsky::prelude::*;

pub fn key_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    //TODO for some reason when I test hurl files with the hurl cli using
    //these escape sequences I get errors. I need to investivate if that is
    //a version issue or if my understanding on this grammar is wrong
    let key_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .to('\\')
                .or(just('#').to('#'))
                .or(just(':').to(':'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t')),
        )
        .or(escaped_unicode_parser())
        .labelled("key-string-escaped-char");

    let key_string_text = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric()
            || c == &'_'
            || c == &'-'
            || c == &'.'
            || c == &'['
            || c == &']'
            || c == &'@'
            || c == &'$'
    })
    .labelled("key-string-text");

    let key_string_content = key_string_text
        .or(key_string_escaped_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("key-string-content");

    let template = template_parser();
    let key_template_part = template
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("key-template");

    let key_string = key_string_content
        .or(key_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k })
        .labelled("key-string");

    key_string
}

pub fn value_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    let value_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .to('\\')
                .or(just('#').to('#'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t')), // .or(escaped_unicode_parser()),
        )
        .or(escaped_unicode_parser())
        .labelled("value-string-escaped-char");

    let template = template_parser();

    let value_string_text =
        filter::<_, _, Simple<char>>(|c: &char| c != &'#' && c != &'\n' && c != &'\\' && c != &'{')
            .labelled("value-string-text");

    let value_string_content = value_string_text
        .or(value_string_escaped_char)
        //opening curly brackes are valid as long as they are not followed by a
        //second curly bracket since two denote the start of a template
        //(this observation isn't explicit in the grammer but was determined
        //by testing requests with hurl)
        .or(just('{').then(just('{').not().rewind()).to('{'))
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("value-string-content");

    let value_template_part = template
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("value-template");

    let value_string = value_template_part
        .or(value_string_content)
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v })
        .labelled("value-string");

    value_string
}

pub fn key_value_parser() -> impl Parser<char, KeyValue, Error = Simple<char>> + Clone {
    let sp = sp_parser();
    let key = key_parser();
    let value = value_parser();

    let key_value = key
        .then_ignore(sp.clone().repeated())
        .then_ignore(just(':'))
        .then_ignore(sp.repeated())
        .then(value)
        .map(|(key, value)| KeyValue { key, value })
        .labelled("key-value");

    key_value
}

#[cfg(test)]
mod value_string_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_value_with_template() {
        let test_str = r#"Bearer {{token}}"#;
        assert_debug_snapshot!(
        value_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "Bearer ",
                    ),
                    Template(
                        Template {
                            expr: Expr {
                                variable: VariableName(
                                    "token",
                                ),
                                filters: [],
                            },
                        },
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_value_with_template_and_extra_whitespace() {
        let test_str = r#"Bearer {{  token  }}"#;
        assert_debug_snapshot!(
        value_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "Bearer ",
                    ),
                    Template(
                        Template {
                            expr: Expr {
                                variable: VariableName(
                                    "token",
                                ),
                                filters: [],
                            },
                        },
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_value_with_possibly_invalid_template() {
        //TODO `{{` denotes the start of a template and `}}` denotes the end of template.
        //However `{ {` is not a valid start and `} }` is not a valid ending of a template
        //This should probably give a diagnostic warning if the user types { { template}}
        //or {{ template} }
        let test_str = r#"Bearer { {token}}"#;
        assert_debug_snapshot!(
        value_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "Bearer { {token}}",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_errors_missing_template_variable() {
        let test_str = r#"Bearer {{}}"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 9..10,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '}',
                    ),
                    label: Some(
                        "template",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_missing_template_variable_with_spaces() {
        let test_str = r#"Bearer {{  }}"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 11..12,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '}',
                    ),
                    label: Some(
                        "template",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_with_unclosed_template() {
        let test_str = r#"Bearer {{token"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 14..14,
                    reason: Unexpected,
                    expected: {
                        Some(
                            'd',
                        ),
                        Some(
                            'r',
                        ),
                        Some(
                            '}',
                        ),
                        Some(
                            'h',
                        ),
                        Some(
                            'n',
                        ),
                        Some(
                            's',
                        ),
                        Some(
                            'c',
                        ),
                        Some(
                            't',
                        ),
                        Some(
                            'u',
                        ),
                        Some(
                            'x',
                        ),
                        Some(
                            'f',
                        ),
                        Some(
                            'j',
                        ),
                    },
                    found: None,
                    label: Some(
                        "template",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_missing_template_variable_with_unclosed_template() {
        let test_str = r#"Bearer {{"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 9..9,
                    reason: Unexpected,
                    expected: {},
                    found: None,
                    label: Some(
                        "template",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_value_with_emoji() {
        let test_str = r#"emoji\u{1F600}"#;
        assert_debug_snapshot!(
        value_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "emoji😀",
                    ),
                ],
            },
        )
        "#,
        );
    }
}

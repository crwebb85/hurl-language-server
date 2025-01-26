use super::primitives::escaped_unicode_parser;
use super::types::{InterpolatedString, InterpolatedStringPart, KeyValue};
use super::{primitives::sp_parser, template::template_parser};
use chumsky::prelude::*;

fn key_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    let key_string_escaped_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('#').to('#'),
            just(':').to(':'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
        )))
        .or(escaped_unicode_parser())
        .labelled("key-string-escaped-char");
    key_string_escaped_char
}

pub fn key_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let key_string_content = choice((
        any().filter(char::is_ascii_alphanumeric),
        one_of("_-.[]@$"),
        key_string_escaped_char_parser(),
    ))
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("key-string-content");

    let key_template_part = template_parser()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("key-template");

    let key_string = choice((key_string_content, key_template_part))
        .repeated()
        .at_least(1)
        .collect::<Vec<InterpolatedStringPart>>()
        .map(|k| InterpolatedString { parts: k })
        .labelled("key-string");

    key_string
}

fn value_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    let value_string_escaped_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('#').to('#'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
        )))
        .or(escaped_unicode_parser())
        .labelled("value-string-escaped-char");
    value_string_escaped_char
}

pub fn value_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let value_string_content = choice((
        none_of("#\n\\{"),
        value_string_escaped_char_parser(),
        //opening curly brackes are valid as long as they are not followed by a
        //second curly bracket since two denote the start of a template
        //(this observation isn't explicit in the grammer but was determined
        just('{').then(just('{').not().rewind()).to('{'),
    ))
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("value-string-content");

    let value_template_part = template_parser()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("value-template");

    let value_string = value_template_part
        .or(value_string_content)
        .repeated()
        .at_least(1)
        .collect::<Vec<InterpolatedStringPart>>()
        .map(|v| InterpolatedString { parts: v })
        .labelled("value-string");

    value_string
}

pub fn key_value_parser<'a>(
) -> impl Parser<'a, &'a str, KeyValue, extra::Err<Rich<'a, char>>> + Clone {
    let key_value = key_parser()
        .padded_by(sp_parser().repeated())
        .then_ignore(just(':').padded_by(sp_parser().repeated()))
        .then(value_parser())
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_value_with_template_and_extra_whitespace() {
        let test_str = r#"Bearer {{  token  }}"#;
        assert_debug_snapshot!(
        value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "Bearer { {token}}",
                        ),
                    ],
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_missing_template_variable() {
        let test_str = r#"Bearer {{}}"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''{'' at 8..9 expected spacing, expr, or something else,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_missing_template_variable_with_spaces() {
        let test_str = r#"Bearer {{  }}"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''}'' at 11..12 expected spacing, or expr,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_with_unclosed_template() {
        let test_str = r#"Bearer {{token"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 14..14 expected ascii alphanumeric char or underscore or dash, spacing, ''c'', ''d'', ''h'', ''t'', ''u'', ''f'', ''j'', ''n'', ''r'', ''s'', ''x'', or ''}'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_missing_template_variable_with_unclosed_template() {
        let test_str = r#"Bearer {{"#;
        assert_debug_snapshot!(
        value_parser().then(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''{'' at 8..9 expected spacing, expr, or something else,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_value_with_emoji() {
        let test_str = r#"emoji\u{1F600}"#;
        assert_debug_snapshot!(
        value_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "emoji😀",
                        ),
                    ],
                },
            ),
            errs: [],
        }
        "#,
        );
    }
}

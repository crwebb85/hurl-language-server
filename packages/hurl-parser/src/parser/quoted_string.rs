use crate::parser::template::template_parser;
use crate::parser::types::{InterpolatedString, InterpolatedStringPart};
use chumsky::prelude::*;

pub fn quoted_string_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone
{
    let quoted_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .or(just('\"').to('\"'))
                .or(just('\\').to('\\'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t'))
                .or(just('u').ignore_then(
                    filter(|c: &char| c.is_digit(16))
                        .repeated()
                        .exactly(4)
                        .collect::<String>()
                        .validate(|digits, span, emit| {
                            char::from_u32(u32::from_str_radix(&digits, 16).unwrap())
                                .unwrap_or_else(|| {
                                    emit(Simple::custom(span, "invalid unicode character"));
                                    '\u{FFFD}' // unicode replacement character
                                })
                        }),
                )),
        )
        .labelled("quoted_string_escaped_char");

    let quoted_str_part = filter::<_, _, Simple<char>>(|c: &char| c != &'"' && c != &'\\')
        .or(quoted_string_escaped_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str);

    let template = template_parser();

    let quoted_template_part = template.map(|t| InterpolatedStringPart::Template(t));

    let quoted_part = quoted_template_part.or(quoted_str_part);

    let quoted_string = just("\"")
        .ignored()
        .then(quoted_part.repeated())
        .then_ignore(just("\""))
        .map(|(_, v)| InterpolatedString { parts: v })
        .labelled("quoted_string");

    quoted_string
}

#[cfg(test)]
mod quoted_string_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_qouted_string() {
        let test_str = "\"gb2312\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "gb2312",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_quoted_empty_string() {
        let test_str = "\"\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r"
        Ok(
            InterpolatedString {
                parts: [],
            },
        )
        ",
        );
    }

    #[test]
    fn it_errors_missing_end_quote() {
        let test_str = "\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 1..1,
                    reason: Unexpected,
                    expected: {
                        Some(
                            '\\',
                        ),
                        Some(
                            '"',
                        ),
                        Some(
                            '{',
                        ),
                    },
                    found: None,
                    label: Some(
                        "quoted_string",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_missing_end_quote_with_contents() {
        let test_str = "\"hello";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 6..6,
                    reason: Unexpected,
                    expected: {
                        Some(
                            '\\',
                        ),
                        Some(
                            '"',
                        ),
                        Some(
                            '{',
                        ),
                    },
                    found: None,
                    label: Some(
                        "quoted_string",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_quoted_string_escaped_quotes() {
        let test_str = "\"\\\"I'm in qoutes\\\"\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "\"I'm in qoutes\"",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_quoted_template() {
        let test_str = "\"{{seperator}}\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Template(
                        Template {
                            expr: Expr {
                                variable: VariableName(
                                    "seperator",
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
    fn it_parses_escape_sequences_in_qouted_string() {
        //TODO figure out if this is the correct handling for escape characters
        let test_str = "\"escapedchars(\\\", \\\\, \\b, \\f, \\r\\n, \\t)\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "escapedchars(\", \\, \u{8}, \u{c}, \r\n, \t)",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_escaped_emoji_in_qouted_string() {
        //TODO figure out if this is the correct handling for unicode characters
        let test_str = "\"escapedemoji(\\u0001\\uF600)\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "escapedemoji(\u{1}\u{f600})",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_escaped_unicode_in_qouted_string() {
        //unicode must include 4 hex digits to be valid. 'H' is not a valid hex digit.
        let test_str = "\"escapedemoji(\\uFFFH)\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 19..20,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        'H',
                    ),
                    label: Some(
                        "quoted_string_escaped_char",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_unicode_length_in_qouted_string() {
        //TODO I would like better error handling in this test case so that
        //it identifies the problem of the escaped unicode having fewer than
        //for digits
        let test_str = "\"\\uFFF\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 6..7,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '"',
                    ),
                    label: Some(
                        "quoted_string_escaped_char",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_escape_char_in_qouted_string() {
        // g is not a valid character for escaping
        let test_str = "\"invalidescapechar:\\g\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 20..21,
                    reason: Unexpected,
                    expected: {
                        Some(
                            'r',
                        ),
                        Some(
                            'n',
                        ),
                        Some(
                            'b',
                        ),
                        Some(
                            '"',
                        ),
                        Some(
                            't',
                        ),
                        Some(
                            '\\',
                        ),
                        Some(
                            'u',
                        ),
                        Some(
                            'f',
                        ),
                    },
                    found: Some(
                        'g',
                    ),
                    label: Some(
                        "quoted_string_escaped_char",
                    ),
                },
            ],
        )
        "#,
        );
    }
}

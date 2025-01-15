use crate::parser::types::{InterpolatedString, InterpolatedStringPart};
use chumsky::prelude::*;

use super::{primitives::escaped_unicode_parser, template::template_parser, types::Template};

/// This exists to decouple the template parser and the quoted string parser
/// since the grammer for the too recursively depend on each other
///
/// # Arguments
///
/// * `template` - the template parser.
///
/// # Returns
/// The quoted string parser based on the given template parser
///
/// ```
pub fn generic_quoted_string_parser<T: Parser<char, Template, Error = Simple<char>> + Clone>(
    template: T,
) -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    let quoted_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .to('\\')
                .or(just('\"').to('\"'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t')),
        )
        .or(escaped_unicode_parser())
        .labelled("quoted_string_escaped_char");

    let quoted_str_part = filter::<_, _, Simple<char>>(|c: &char| c != &'"' && c != &'\\')
        .or(quoted_string_escaped_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str);

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

pub fn quoted_string_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone
{
    let template_parser = template_parser();
    generic_quoted_string_parser(template_parser)
}

#[cfg(test)]
mod quoted_string_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_quoted_string() {
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
    fn it_parses_escape_sequences_in_quoted_string() {
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
    fn it_parses_escaped_emoji_in_quoted_string() {
        let test_str = "\"escapedemoji(\\u{0001}\\u{F600})\"";
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
    fn it_errors_invalid_escaped_unicode_in_quoted_string() {
        //unicode must include 4 hex digits to be valid. 'H' is not a valid hex digit.
        let test_str = "\"escapedemoji(\\u{FFFH})\"";
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
                            '}',
                        ),
                    },
                    found: Some(
                        'H',
                    ),
                    label: Some(
                        "escaped-unicode-char",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_escape_char_in_quoted_string() {
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
                        "escaped-unicode-char",
                    ),
                },
            ],
        )
        "#,
        );
    }
}

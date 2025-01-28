use crate::parser::types::{InterpolatedString, InterpolatedStringPart};
use chumsky::prelude::*;

use super::{primitives::escaped_unicode_parser, template::template_parser, types::Template};

fn quoted_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    let quoted_string_escaped_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('\"').to('\"'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
        )))
        .or(escaped_unicode_parser())
        .labelled("quoted_string_escaped_char");
    quoted_string_escaped_char.boxed()
}

fn quoted_str_part_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedStringPart, extra::Err<Rich<'a, char>>> + Clone {
    let quoted_str_part = choice((quoted_string_escaped_char_parser(), none_of("\"\\")))
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str);
    quoted_str_part.boxed()
}

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
pub fn generic_quoted_string_parser<
    'a,
    T: Parser<'a, &'a str, Template, extra::Err<Rich<'a, char>>> + Clone,
>(
    template: T,
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let template_part = template.map(InterpolatedStringPart::Template);

    let parts = choice((template_part, quoted_str_part_parser()))
        .repeated()
        .collect::<Vec<InterpolatedStringPart>>()
        .map(|v| InterpolatedString { parts: v });

    let quoted_string = parts
        .delimited_by(just("\""), just("\""))
        .labelled("quoted_string");

    quoted_string
}

pub fn quoted_string_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let template_parser = template_parser();
    generic_quoted_string_parser(template_parser).boxed()
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
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "gb2312",
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
    fn it_parses_quoted_empty_string() {
        let test_str = "\"\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [],
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_errors_missing_end_quote() {
        let test_str = "\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: None,
            errs: [
                found end of input at 1..1 expected template, quoted_string_escaped_char, something else, or ''"'',
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_missing_end_quote_with_contents() {
        let test_str = "\"hello";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: None,
            errs: [
                found end of input at 6..6 expected quoted_string_escaped_char, something else, template, or ''"'',
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_quoted_string_escaped_quotes() {
        let test_str = "\"\\\"I'm in quotes\\\"\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "\"I'm in quotes\"",
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
    fn it_parses_quoted_template() {
        let test_str = "\"{{seperator}}\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_escape_sequences_in_quoted_string() {
        let test_str = r#""escapedchars(\", \\, \b, \f, \r\n, \t)""#;
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "escapedchars(\", \\, \u{8}, \u{c}, \r\n, \t)",
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
    fn it_parses_escaped_emoji_in_quoted_string() {
        let test_str = "\"escapedemoji(\\u{0001}\\u{F600})\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "escapedemoji(\u{1}\u{f600})",
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
    fn it_errors_invalid_escaped_unicode_in_quoted_string() {
        //unicode must include 4 hex digits to be valid. 'H' is not a valid hex digit.
        let test_str = "\"escapedemoji(\\u{FFFH})\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''H'' at 20..21 expected digit, or ''}'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_invalid_escape_char_in_quoted_string() {
        // g is not a valid character for escaping
        let test_str = "\"invalidescapechar:\\g\"";
        assert_debug_snapshot!(
        quoted_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: None,
            errs: [
                found ''g'' at 20..21 expected ''\\'', ''"'', ''b'', ''f'', ''n'', ''r'', ''t'', or ''u'',
            ],
        }
        "#,
        );
    }
}

use chumsky::prelude::*;

use super::{
    primitives::escaped_unicode_parser,
    template::template_parser,
    types::{InterpolatedString, InterpolatedStringPart},
};

fn oneline_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    let oneline_string_escaped_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('#').to('#'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('`').to('`'),
            //TODO off-spec the actual parser allows escaping \n \r and \t
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
        )))
        .or(escaped_unicode_parser())
        .labelled("oneline-string-escaped-char");
    oneline_string_escaped_char.boxed()
}

pub fn oneline_string_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let oneline_string_content = choice((
        none_of("#\n\\{`"),
        oneline_string_escaped_char_parser(),
        //opening curly brackes are valid as long as they are not followed by a
        //second curly bracket since two denote the start of a template
        //(this observation isn't explicit in the grammer but was determined
        just('{').then(just('{').not().rewind()).to('{'),
    ))
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("oneline-string-content");

    let oneline_template_part = template_parser()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("oneline-string-template");

    let oneline_string_parts = choice((oneline_template_part, oneline_string_content))
        .repeated()
        .collect::<Vec<InterpolatedStringPart>>()
        .map(|v| InterpolatedString { parts: v });

    oneline_string_parts
        .delimited_by(just("`"), just("`"))
        .labelled("oneline-string")
        .boxed()
}

#[cfg(test)]
mod oneline_string_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_string() {
        let test_str = r#"`test`"#;
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "test",
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
    fn it_parses_oneline_empty_string() {
        let test_str = "``";
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
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
    fn it_errors_missing_end_backtick() {
        let test_str = "`";
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 1..1 expected oneline-string-template, oneline-string-content, or ''`'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_missing_end_backtick_with_contents() {
        let test_str = "`hello";
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 6..6 expected something else, oneline-string-escaped-char, ''{'', oneline-string-template, oneline-string-content, or ''`'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_oneline_string_escaped_backtics() {
        let test_str = r#"`\`I'm in backtick quotes\``"#;
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "`I'm in backtick quotes`",
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
    fn it_parses_oneline_string_template() {
        let test_str = "`{{seperator}}`";
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
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
        let test_str = r#"`escapedchars(\`, \#, \\, \b, \f, \r\n, \t)`"#;
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "escapedchars(`, #, \\, \u{8}, \u{c}, \r\n, \t)",
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
    fn it_parses_escaped_emoji_in_oneline_string() {
        let test_str = r#"`escapedemoji(\u{0001}\u{F600})`"#;
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
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
    fn it_errors_invalid_escaped_unicode_in_oneline_string() {
        //unicode must include 4 hex digits to be valid. 'H' is not a valid hex digit.
        let test_str = r#"`escapedemoji(\u{FFFH})`"#;
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
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
    fn it_errors_invalid_escape_char_in_oneline_string() {
        // g is not a valid character for escaping
        let test_str = r#"`invalidescapechar:\g`"#;
        assert_debug_snapshot!(
        oneline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''g'' at 20..21 expected ''\\'', ''#'', ''b'', ''f'', ''`'', ''n'', ''r'', ''t'', or ''u'',
            ],
        }
        ",
        );
    }
}

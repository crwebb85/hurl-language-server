use chumsky::prelude::*;

use super::{
    primitives::{alphanumeric_parser, escaped_unicode_parser, lt_parser, sp_parser, todo_parser},
    request_section::oneline_file_parser,
    template::template_parser,
    types::{Body, Bytes, InterpolatedString, InterpolatedStringPart, Json, MultiLineString},
};

pub fn bytes_parser<'a>() -> impl Parser<'a, &'a str, Bytes, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        json_value_parser().map(Bytes::JsonValue),
        //xml_parser().map(Bytes::Xml), //TODO when hurl implements syntax for xml bytes
        multiline_string_parser().map(Bytes::MultilineString),
        oneline_string_parser().map(Bytes::OneLineString),
        oneline_base64_parser().map(Bytes::OneLineBase64),
        oneline_file_parser().map(Bytes::OneLineFile),
        oneline_hex_parser().map(Bytes::OneLineHex),
    ))
    .labelled("bytes")
    .boxed()
}

fn json_value_parser<'a>() -> impl Parser<'a, &'a str, Json, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser().map(|_| Json::Invalid).boxed()
}

fn multiline_string_parser<'a>(
) -> impl Parser<'a, &'a str, MultiLineString, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser()
        .map(|_| MultiLineString::Json(InterpolatedString { parts: vec![] }))
        .boxed()
}

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

fn oneline_base64_parser<'a>(
) -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    just("base64,")
        .padded_by(sp_parser().repeated())
        .ignore_then(
            choice((
                alphanumeric_parser(),
                //TODO off-spec \t is not in the spec but is in the official parser and I verified
                //it works
                //TODO \n is in the spec and in the official parser but when I try to use a hurl
                //file with it it allways errors saying "expecting ';'"
                one_of("+-=\n \t"),
            ))
            .repeated()
            .collect::<String>(),
        )
        .then_ignore(just(";"))
}

fn oneline_hex_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone
{
    just("hex,")
        .ignore_then(
            text::digits(16)
                .to_slice()
                .padded_by(sp_parser().repeated())
                .map(|s: &str| s.to_string()),
        )
        .then_ignore(just(";"))
}

pub fn body_parser<'a>() -> impl Parser<'a, &'a str, Body, extra::Err<Rich<'a, char>>> + Clone {
    bytes_parser()
        .then_ignore(lt_parser())
        .map(|bytes| Body { bytes })
        .labelled("body")
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

#[cfg(test)]
mod oneline_base64_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_base64_string() {
        let test_str = r#"base64, VGhpcyBpcyBhIHRlc3Q=;"#;
        assert_debug_snapshot!(
        oneline_base64_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "VGhpcyBpcyBhIHRlc3Q=",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_base64_string_with_extra_spacing() {
        let test_str = r#"base64,  VGhpc  yBpcyBhIHRl  c3Q =  ;"#;
        //TODO I'm not sure if it would be better to remove the whitespace
        //from the parsed base 64 value or leave it alone. Both are valid.
        assert_debug_snapshot!(
        oneline_base64_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "VGhpc  yBpcyBhIHRl  c3Q =  ",
            ),
            errs: [],
        }
        "#,
        );
    }
}

#[cfg(test)]
mod oneline_hex_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_hex_string() {
        let test_str = r#"hex, 2AFA;"#;
        assert_debug_snapshot!(
        oneline_hex_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "2AFA",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_hex_string_with_extra_spaces() {
        let test_str = r#"hex,   2AFA  ;"#;
        assert_debug_snapshot!(
        oneline_hex_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "2AFA",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_hex_lowercase_string() {
        let test_str = r#"hex, 2afa;"#;
        assert_debug_snapshot!(
        oneline_hex_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "2afa",
            ),
            errs: [],
        }
        "#,
        );
    }
}

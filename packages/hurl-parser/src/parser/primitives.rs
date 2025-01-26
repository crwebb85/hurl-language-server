use chumsky::prelude::*;

use super::types::Lt;

/// A parser that never matches. Used as a placeholder for parsers that
/// I haven't yet implemented but plan to
pub fn todo_parser<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    any().filter(|_| false)
}

pub fn sp_parser<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    one_of("\t ").labelled("spacing")
}

pub fn comment_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone
{
    let comment = just('#')
        .ignore_then(none_of('\n').repeated().collect::<String>())
        .labelled("comment");
    comment
}

pub fn lt_not_end_parser<'a>() -> impl Parser<'a, &'a str, Lt, extra::Err<Rich<'a, char>>> + Clone {
    sp_parser()
        .repeated()
        .ignore_then(comment_parser().or_not())
        .then_ignore(text::newline())
        .map(|comment| Lt { comment })
        .labelled("line terminator")
}

pub fn lt_at_end_parser<'a>() -> impl Parser<'a, &'a str, Lt, extra::Err<Rich<'a, char>>> + Clone {
    sp_parser()
        .repeated()
        .ignore_then(comment_parser().or_not())
        .then_ignore(end())
        .map(|comment| Lt { comment })
        .labelled("line terminator")
}

pub fn lt_parser<'a>() -> impl Parser<'a, &'a str, Vec<Lt>, extra::Err<Rich<'a, char>>> + Clone {
    //TODO this looks really stupid but since the old lt consumes the end() token doing
    //lt.repeated() would cause a crash. I want to refactor this so it doesn't look so stupid
    choice((
        lt_not_end_parser()
            .repeated()
            .at_least(1)
            .collect::<Vec<_>>()
            .then(lt_at_end_parser().or_not())
            .map(|(mut lts, optional_lt)| {
                match optional_lt {
                    Some(l) => lts.push(l),
                    None => (),
                };
                lts
            }),
        lt_at_end_parser().map(|lt| vec![lt]),
    ))
}

pub fn escaped_unicode_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    just('\\')
        .ignored()
        .then_ignore(just('u'))
        .then_ignore(just('{'))
        .then(text::digits(16).to_slice().validate(|digits, e, emitter| {
            char::from_u32(u32::from_str_radix(digits, 16).unwrap()).unwrap_or_else(|| {
                emitter.emit(Rich::custom(e.span(), "invalid unicode character"));
                '\u{FFFD}' // unicode replacement character
            })
        }))
        .then_ignore(just('}'))
        .map(|(_, v)| v)
        .labelled("escaped-unicode-char")
}

#[cfg(test)]
mod sp_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_space() {
        let test_str = " ";
        assert_debug_snapshot!(
        sp_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                ' ',
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_tab() {
        let test_str = "\t";
        assert_debug_snapshot!(
        sp_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                '\t',
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_errors_linefeed() {
        let test_str = "\n";
        assert_debug_snapshot!(
        sp_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''\n'' at 0..1 expected spacing,
            ],
        }
        ",
        );
    }
}

#[cfg(test)]
mod lt_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_linefeed() {
        let test_str = "\n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
                    Lt {
                        comment: None,
                    },
                    Lt {
                        comment: None,
                    },
                ],
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_linefeed_and_carriage_return() {
        let test_str = "\r\n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
                    Lt {
                        comment: None,
                    },
                    Lt {
                        comment: None,
                    },
                ],
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_end_of_file() {
        let test_str = "";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
                    Lt {
                        comment: None,
                    },
                ],
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_end_of_file_with_comment() {
        let test_str = "  # this is another comment";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
                    Lt {
                        comment: Some(
                            " this is another comment",
                        ),
                    },
                ],
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_for_no_lineending() {
        let test_str = "not a line ending";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''n'' at 0..1 expected line terminator,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_lineending_without_comment() {
        let test_str = "   \n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
                    Lt {
                        comment: None,
                    },
                    Lt {
                        comment: None,
                    },
                ],
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_lineending_with_comment() {
        let test_str = "   # this is a comment\n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
                    Lt {
                        comment: Some(
                            " this is a comment",
                        ),
                    },
                    Lt {
                        comment: None,
                    },
                ],
            ),
            errs: [],
        }
        "#,
        );
    }
}

#[cfg(test)]
mod unicode_parser_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_emoji() {
        let test_str = r#"\u{1F600}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
                '😀',
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_errors_invalid_unicode() {
        let test_str = r#"\u{1F6000}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
                '�',
            ),
            errs: [
                invalid unicode character at 3..9,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_unicode_missing_bracket() {
        let test_str = r#"\u1F6000"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''1'' at 2..3 expected ''{'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_short_unicode() {
        let test_str = r#"\u{1F6}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
                'Ƕ',
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_errors_invalid_hex_digit_in_unicode() {
        let test_str = r#"\u{FFFH}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''H'' at 6..7 expected digit, or ''}'',
            ],
        }
        ",
        );
    }
}

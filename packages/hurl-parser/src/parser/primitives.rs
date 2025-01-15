use chumsky::prelude::*;

use super::types::Lt;

pub fn sp_parser() -> impl Parser<char, char, Error = Simple<char>> + Clone {
    filter(|c: &char| c.is_whitespace() && (c == &'\t' || c == &' ')).labelled("Space or tab")
}

pub fn lt_parser() -> impl Parser<char, Lt, Error = Simple<char>> + Clone {
    let sp = sp_parser();

    let comment = just('#')
        .then(
            filter::<_, _, Simple<char>>(|c| c != &'\n')
                .repeated()
                .at_least(1)
                .collect::<String>(),
        )
        .map(|(_, comment)| comment)
        .labelled("comment");

    sp.repeated()
        .ignored()
        .then(comment.or_not())
        .then_ignore(text::newline().or(end()))
        .map(|(_, comment)| Lt { comment })
        .labelled("line terminator")
}

pub fn escaped_unicode_parser() -> impl Parser<char, char, Error = Simple<char>> + Clone {
    just('\\')
        .then(just('u'))
        .then(just('{'))
        .then(
            filter(|c: &char| c.is_digit(16))
                .repeated()
                .at_least(1)
                .collect::<String>()
                .validate(|digits, span, emit| {
                    char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(|| {
                        emit(Simple::custom(span, "invalid unicode character"));
                        '\u{FFFD}' // unicode replacement character
                    })
                }),
        )
        .then(just('}'))
        .map(|((_, u), _)| u)
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
        Ok(
            ' ',
        )
        ",
        );
    }

    #[test]
    fn it_parses_tab() {
        let test_str = "\t";
        assert_debug_snapshot!(
        sp_parser().parse(test_str),
            @r"
        Ok(
            '\t',
        )
        ",
        );
    }

    #[test]
    fn it_errors_linefeed() {
        let test_str = "\n";
        assert_debug_snapshot!(
        sp_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '\n',
                    ),
                    label: Some(
                        "Space or tab",
                    ),
                },
            ],
        )
        "#,
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
        Ok(
            Lt {
                comment: None,
            },
        )
        ",
        );
    }

    #[test]
    fn it_parses_linefeed_and_carriage_return() {
        let test_str = "\r\n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        Ok(
            Lt {
                comment: None,
            },
        )
        ",
        );
    }

    #[test]
    fn it_parses_end_of_file() {
        let test_str = "";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        Ok(
            Lt {
                comment: None,
            },
        )
        ",
        );
    }

    #[test]
    fn it_errors_for_no_lineending() {
        let test_str = "not a line ending";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {
                        Some(
                            '\r',
                        ),
                        Some(
                            '\n',
                        ),
                        None,
                        Some(
                            '#',
                        ),
                    },
                    found: Some(
                        'n',
                    ),
                    label: Some(
                        "line terminator",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_lineending_without_comment() {
        let test_str = "   \n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r"
        Ok(
            Lt {
                comment: None,
            },
        )
        ",
        );
    }

    #[test]
    fn it_parses_lineending_with_comment() {
        let test_str = "   # this is a comment\n";
        assert_debug_snapshot!(
        lt_parser().parse(test_str),
            @r#"
        Ok(
            Lt {
                comment: Some(
                    " this is a comment",
                ),
            },
        )
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
        Ok(
            '😀',
        )
        ",
        );
    }

    #[test]
    fn it_errors_invalid_unicode() {
        let test_str = r#"\u{1F6000}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 3..9,
                    reason: Custom(
                        "invalid unicode character",
                    ),
                    expected: {},
                    found: None,
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
    fn it_errors_unicode_missing_bracket() {
        let test_str = r#"\u1F6000"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 2..3,
                    reason: Unexpected,
                    expected: {
                        Some(
                            '{',
                        ),
                    },
                    found: Some(
                        '1',
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
    fn it_parses_short_unicode() {
        let test_str = r#"\u{1F6}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r"
        Ok(
            'Ƕ',
        )
        ",
        );
    }

    #[test]
    fn it_errors_invalid_hex_digit_in_unicode() {
        let test_str = r#"\u{FFFH}"#;
        assert_debug_snapshot!(
        escaped_unicode_parser().then_ignore(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 6..7,
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
}

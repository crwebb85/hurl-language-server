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

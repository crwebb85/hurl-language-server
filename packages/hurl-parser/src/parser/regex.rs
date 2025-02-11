use chumsky::prelude::*;

use super::types::{InterpolatedString, Regex};

pub fn regex_literal_parser<'a>(
) -> impl Parser<'a, &'a str, Regex, extra::Err<Rich<'a, char>>> + Clone {
    let regex_escaped_char = just('\\')
        .ignore_then(choice((just('/').to('/'), none_of('\n').rewind().to('\\'))))
        .labelled("regex_escaped_char")
        .boxed();

    let regex_content = choice((regex_escaped_char, none_of("\n\\/")))
        .repeated()
        .collect::<String>();

    let regex = regex_content
        .delimited_by(just("/"), just("/"))
        .labelled("regex")
        .map(Regex::Literal)
        .boxed();
    regex
}

pub fn regex_parser<
    'a,
    T: Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone + 'a,
>(
    quoted_string: T,
) -> impl Parser<'a, &'a str, Regex, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        regex_literal_parser(),
        quoted_string.map(Regex::Interpolated),
    ))
    .labelled("regex-or-interpolated-regex")
    .boxed()
}

#[cfg(test)]
mod regex_tests {

    use super::*;
    use crate::parser::quoted_string::quoted_string_parser;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_regex_literal_empty_string() {
        let test_str = "//";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        regex_parser(quoted_string).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Literal(
                    "",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_interpolated_regex_empty_string() {
        let test_str = r#""""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        regex_parser(quoted_string).parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Interpolated(
                    InterpolatedString {
                        parts: [],
                    },
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_simple_regex_literal() {
        let test_str = r#"/\d{10}/"#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        regex_parser(quoted_string).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Literal(
                    "\\d{10}",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_simple_interpolated_regex_without_templates() {
        let test_str = r#""\\d{10}""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        regex_parser(quoted_string).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Interpolated(
                    InterpolatedString {
                        parts: [
                            Str(
                                "\\d{10}",
                            ),
                        ],
                    },
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_simple_interpolated_regex_with_template_inside_curly_brackets() {
        let test_str = r#""\\d{ {{count}} }""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        regex_parser(quoted_string).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Interpolated(
                    InterpolatedString {
                        parts: [
                            Str(
                                "\\d{ ",
                            ),
                            Template(
                                Template {
                                    expr: Expr {
                                        variable: VariableName(
                                            "count",
                                        ),
                                        filters: [],
                                    },
                                },
                            ),
                            Str(
                                " }",
                            ),
                        ],
                    },
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_simple_interpolated_regex_with_template_inside_curly_brackets_without_spaces() {
        let test_str = r#""\\d{{{count}}}""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        regex_parser(quoted_string).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''{'' at 5..6 expected spacing, expr, or something else,
            ],
        }
        ",
        );
    }
}

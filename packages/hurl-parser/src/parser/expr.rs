use chumsky::{error::Simple, prelude::{choice, filter, just}, text, Parser};

use super::{primitives::sp_parser, types::{Expr, ExprValue, FilterFunction, InterpolatedString}};

pub fn variable_name_parser() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    let variable_name = filter::<_, _, Simple<char>>(char::is_ascii_alphabetic)
        .then(
            filter::<_, _, Simple<char>>(|c: &char| {
                c.is_ascii_alphanumeric() || c == &'_' || c == &'-'
            })
            .repeated()
            .collect::<String>(),
        )
        .map(|(c, chars)| format!("{}{}", c, chars)).labelled("variable_name");
    variable_name
}

pub fn expr_parser<T: Parser<char, InterpolatedString, Error = Simple<char>> + Clone>(
    quoted_string: T,
    ) -> impl Parser<char, Expr, Error = Simple<char>> + Clone {

    let sp = sp_parser();
    let variable_name = variable_name_parser();
    let expr_function = choice::<_, Simple<char>>([
        text::keyword("getEnv").to(ExprValue::FunctionName("getEnv".to_owned())), 
        text::keyword("newDate").to(ExprValue::FunctionName("newDate".to_owned())),
        text::keyword("newUuid").to(ExprValue::FunctionName("newUuid".to_owned()))
    ]);

    let expr_variable = expr_function.or(variable_name.map(ExprValue::VariableName));


    let decode_filter_function = just("decode")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Decode { encoding: s});

    let format_filter_function = just("format")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Format { fmt: s});

    let jsonpath_filter_function = just("jsonpath")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::JsonPath { expr: s });

    let nth_filter_function = just("nth")
        .then_ignore(sp.clone())
        .then(text::int(10))
        .map(|(_, n)| FilterFunction::Nth { 
            nth: n.parse::<u64>()
                .expect("TODO implement error recovery for invalid integers used in the Nth filter function argument") 
        });

    let regex_filter_function = just("regex")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Regex { value: s });

    let split_filter_function = just("split")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Split { sep: s });
        
    let replace_filter_function = just("replace")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|((_, old), new)| FilterFunction::Replace { old_value: old, new_value: new });

    let todate_filter_function = just("toDate")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::ToDate { fmt: s });

    let xpath_filter_function = just("xpath")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::XPath { expr: s });

    //TODO detect type errors between inputs and outputs of filters
    let filter_function = choice::<_, Simple<char>>([
        just("count").to(FilterFunction::Count), 
        just("daysAfterNow").to(FilterFunction::DaysAfterNow),
        just("daysBeforeNow").to(FilterFunction::DaysBeforeNow),
        just("htmlEscape").to(FilterFunction::HtmlEscape),
        just("htmlUnescape").to(FilterFunction::HtmlUnescape),
        just("toFloat").to(FilterFunction::ToFloat),
        just("toInt").to(FilterFunction::ToInt),
        just("urlDecode").to(FilterFunction::UrlDecode),
        just("urlEncode").to(FilterFunction::UrlEncode),
    ]).or(decode_filter_function)
    .or(format_filter_function)
    .or(jsonpath_filter_function)
    .or(nth_filter_function)
    .or(regex_filter_function)
    .or(split_filter_function)
    .or(replace_filter_function)
    .or(todate_filter_function)
    .or(xpath_filter_function);

    let filters = filter_function.separated_by(sp.clone().repeated());

    let expr = expr_variable
    .then_ignore(sp.clone().repeated())
    .then(filters)
    .map( |(expr_var, filter_funcs)| Expr {
        variable: expr_var,
        filters: filter_funcs
    }).labelled("expr");

    expr
}

#[cfg(test)]
mod variable_name_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_variable_name_in_expr() {
        let test_str = "key";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        Ok(
            "key",
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_dashed_variable() {
        let test_str = "api-key";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        Ok(
            "api-key",
        )
        "#,
        );
    }

    #[test]
    fn it_parses_expr_with_underscore_variable() {
        let test_str = "api_key";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        Ok(
            "api_key",
        )
        "#,
        );
    }

    #[test]
    fn it_errors_expr_with_number_starting_variable_name() {
        let test_str = "1";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '1',
                    ),
                    label: Some(
                        "variable_name",
                    ),
                },
            ],
        )
        "#,
        );
    }


    #[test]
    fn it_errors_expr_with_dash_starting_variable_name() {
        let test_str = "-";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '-',
                    ),
                    label: Some(
                        "variable_name",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_expr_with_underscore_starting_variable_name() {
        let test_str = "_";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '_',
                    ),
                    label: Some(
                        "variable_name",
                    ),
                },
            ],
        )
        "#,
        );
    }

}

#[cfg(test)]
mod expr_tests {

    use crate::parser::quoted_string::quoted_string_parser;

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_variable_name_in_expr() {
        let test_str = "key";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "key",
                ),
                filters: [],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_dashed_variable() {
        let test_str = "api-key";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "api-key",
                ),
                filters: [],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_expr_with_underscore_variable() {
        let test_str = "api_key";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "api_key",
                ),
                filters: [],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_errors_expr_with_number_variable() {
        let test_str = "1";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '1',
                    ),
                    label: Some(
                        "variable_name",
                    ),
                },
            ],
        )
        "#,
        );
    }



    #[test]
    fn it_parses_expr_with_decode_filter() {
        let test_str = "api_key decode \"gb2312\"";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "api_key",
                ),
                filters: [
                    Decode {
                        encoding: InterpolatedString {
                            parts: [
                                Str(
                                    "gb2312",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_format_filter() {
        let test_str = "creation_date format \"%a, %d %b %Y %H:%M:%S\"";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "creation_date",
                ),
                filters: [
                    Format {
                        fmt: InterpolatedString {
                            parts: [
                                Str(
                                    "%a, %d %b %Y %H:%M:%S",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_jsonpath_filter() {
        let test_str = "input_data jsonpath \"$[0].last_name\"";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "input_data",
                ),
                filters: [
                    JsonPath {
                        expr: InterpolatedString {
                            parts: [
                                Str(
                                    "$[0].last_name",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_nth_filter() {
        let test_str = "input_data jsonpath \"$[0].names\" nth 2";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "input_data",
                ),
                filters: [
                    JsonPath {
                        expr: InterpolatedString {
                            parts: [
                                Str(
                                    "$[0].names",
                                ),
                            ],
                        },
                    },
                    Nth {
                        nth: 2,
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_regex_filter() {
        let test_str = r#"id regex "\\d{10}""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "id",
                ),
                filters: [
                    Regex {
                        value: InterpolatedString {
                            parts: [
                                Str(
                                    "\\d{10}",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_split_filter() {
        let test_str = r#"names split ", ""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "names",
                ),
                filters: [
                    Split {
                        sep: InterpolatedString {
                            parts: [
                                Str(
                                    ", ",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_replace_filter() {
        let test_str = r#"names replace "; " ",""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "names",
                ),
                filters: [
                    Replace {
                        old_value: InterpolatedString {
                            parts: [
                                Str(
                                    "; ",
                                ),
                            ],
                        },
                        new_value: InterpolatedString {
                            parts: [
                                Str(
                                    ",",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_to_date_filter() {
        let test_str = "creation_date toDate \"%a, %d %b %Y %H:%M:%S\"";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "creation_date",
                ),
                filters: [
                    ToDate {
                        fmt: InterpolatedString {
                            parts: [
                                Str(
                                    "%a, %d %b %Y %H:%M:%S",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_expr_with_xpath_filter() {
        let test_str = r#"document xpath "string(//div)""#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "document",
                ),
                filters: [
                    XPath {
                        expr: InterpolatedString {
                            parts: [
                                Str(
                                    "string(//div)",
                                ),
                            ],
                        },
                    },
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_count_filter() {
        let test_str = r#"response jsonpath "$.names" count"#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "response",
                ),
                filters: [
                    JsonPath {
                        expr: InterpolatedString {
                            parts: [
                                Str(
                                    "$.names",
                                ),
                            ],
                        },
                    },
                    Count,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_days_after_now_filter() {
        let test_str = "expiration_date daysAfterNow";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "expiration_date",
                ),
                filters: [
                    DaysAfterNow,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_days_before_now_filter() {
        let test_str = "expiration_date daysBeforeNow";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "expiration_date",
                ),
                filters: [
                    DaysBeforeNow,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_html_escape_filter() {
        let test_str = "document htmlEscape";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "document",
                ),
                filters: [
                    HtmlEscape,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_html_unescape_filter() {
        let test_str = "escaped_document htmlUnescape";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "escaped_document",
                ),
                filters: [
                    HtmlUnescape,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_to_float_filter() {
        let test_str = "inflation_rate toFloat";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "inflation_rate",
                ),
                filters: [
                    ToFloat,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_to_int_filter() {
        let test_str = "id toInt";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "id",
                ),
                filters: [
                    ToInt,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_url_decode_filter() {
        let test_str = "encoded_url urlDecode";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "encoded_url",
                ),
                filters: [
                    UrlDecode,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_url_encode_filter() {
        let test_str = "url urlEncode";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "url",
                ),
                filters: [
                    UrlEncode,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_extra_space_between_variable_and_filter() {
        let test_str = "url  urlEncode";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "url",
                ),
                filters: [
                    UrlEncode,
                ],
            },
        )
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_extra_space_between_filters() {
        let test_str = r#"response jsonpath "$.names"   count"#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "response",
                ),
                filters: [
                    JsonPath {
                        expr: InterpolatedString {
                            parts: [
                                Str(
                                    "$.names",
                                ),
                            ],
                        },
                    },
                    Count,
                ],
            },
        )
        "#,
        );

    }

}

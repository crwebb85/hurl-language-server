use chumsky::prelude::*;

use super::{primitives::sp_parser, types::{Expr, ExprValue, FilterFunction, InterpolatedString}};


pub fn alphabetic_parser<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {

        let ascii_alphabetic_char = custom::<'a, _, &'a str, char, extra::Err<Rich<'a, char>>>(|inp| {
            
            let before = inp.cursor();
            let c = inp.next();
            let span = inp.span_since(&before);
            match c {
                Some(c) => if c.is_ascii_alphabetic() {
                    Ok(c)
                } else {
                    Err(Rich::custom(span, format!("expected an ascii alphabetic char but found {}", c)))
                },
                None => Err(Rich::custom(span, format!("expected an ascii alphabetic char but found end"))),
            }
            
        });
        ascii_alphabetic_char
        .labelled("ascii-alphabetic-char")
        .boxed()
}


pub fn alphanumeric_parser<'a>() -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {

        let ascii_alphanumeric_char = custom::<'a, _, &'a str, char, extra::Err<Rich<'a, char>>>(|inp| {
            let before = inp.cursor();
            let c = inp.next();
            let span = inp.span_since(&before);
            
            match c {
                Some(c) => if c.is_ascii_alphanumeric() {
                    Ok(c)
                } else {
                    Err(Rich::custom(span, format!("expected an ascii alphanumeric char but found {}", c)))
                },
                None => Err(Rich::custom(span, format!("expected an ascii alphanumeric char but found end"))),
            }
        });
        ascii_alphanumeric_char
        .labelled("ascii-alphanumeric-char")
        .boxed()
}

pub fn variable_name_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {

    let variable_name = alphabetic_parser()
        .labelled("ascii alphabetic char")
        .then(
            choice((
                    one_of("_-"),
                    alphanumeric_parser()
            ))
            .labelled("ascii alphanumeric char or underscore or dash")
            .repeated()
            .to_slice()
        )
        .to_slice()
        .map(ToString::to_string);

    variable_name
        .labelled("variable-name")
        .boxed()
}

pub fn expr_parser<'a, T: Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone>(
    quoted_string: T,
) -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> + Clone {

    let expr_function = choice((
        text::keyword("getEnv").to(ExprValue::FunctionName("getEnv".to_owned())), 
        text::keyword("newDate").to(ExprValue::FunctionName("newDate".to_owned())),
        text::keyword("newUuid").to(ExprValue::FunctionName("newUuid".to_owned()))
    ));

    let expr_variable = expr_function.or(variable_name_parser().map(ExprValue::VariableName));

    let decode_filter_function = just("decode")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::Decode { encoding: s});

    let format_filter_function = just("format")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::Format { fmt: s});

    let jsonpath_filter_function = just("jsonpath")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::JsonPath { expr: s });

    let nth_filter_function = just("nth")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .then(text::int(10))
        .map(|(_, n)| FilterFunction::Nth { 
            nth: n.parse::<u64>()
                .expect("TODO implement error recovery for invalid integers used in the Nth filter function argument") 
        });

    let regex_filter_function = just("regex")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::Regex { value: s });

    let split_filter_function = just("split")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::Split { sep: s });
        
    let replace_filter_function = just("replace")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .then_ignore(sp_parser().repeated().at_least(1))
        .then(quoted_string.clone())
        .map(|(old, new)| FilterFunction::Replace { old_value: old, new_value: new });

    let todate_filter_function = just("toDate")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::ToDate { fmt: s });

    let xpath_filter_function = just("xpath")
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .ignore_then(quoted_string.clone())
        .map(|s| FilterFunction::XPath { expr: s });

    //TODO detect type errors between inputs and outputs of filters
    let filters = choice((
        just("count").to(FilterFunction::Count), 
        just("daysAfterNow").to(FilterFunction::DaysAfterNow),
        just("daysBeforeNow").to(FilterFunction::DaysBeforeNow),
        just("htmlEscape").to(FilterFunction::HtmlEscape),
        just("htmlUnescape").to(FilterFunction::HtmlUnescape),
        just("toFloat").to(FilterFunction::ToFloat),
        just("toInt").to(FilterFunction::ToInt),
        just("urlDecode").to(FilterFunction::UrlDecode),
        just("urlEncode").to(FilterFunction::UrlEncode),
        decode_filter_function,
        format_filter_function,
        jsonpath_filter_function,
        nth_filter_function,
        regex_filter_function,
        split_filter_function,
        replace_filter_function,
        todate_filter_function,
        xpath_filter_function,
    )).separated_by(sp_parser().repeated().at_least(1))
        .collect::<Vec<FilterFunction>>();

    let expr = expr_variable
    .padded_by(sp_parser().repeated())
    .then(filters)
    .map( |(expr_var, filter_funcs)| Expr {
        variable: expr_var,
        filters: filter_funcs
    })
    .labelled("expr");

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
        ParseResult {
            output: Some(
                "key",
            ),
            errs: [],
        }
        "#,
        );
    }


    #[test]
    fn it_parses_expr_with_dashed_variable() {
        let test_str = "api-key";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "api-key",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_expr_with_underscore_variable() {
        let test_str = "api_key";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "api_key",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_expr_with_number_starting_variable_name() {
        let test_str = "1";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 0..1 expected variable-name,
            ],
        }
        ",
        );
    }


    #[test]
    fn it_errors_expr_with_dash_starting_variable_name() {
        let test_str = "-";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 0..1 expected variable-name,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_expr_with_underscore_starting_variable_name() {
        let test_str = "_";
        assert_debug_snapshot!(
        variable_name_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 0..1 expected variable-name,
            ],
        }
        ",
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "key",
                    ),
                    filters: [],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "api-key",
                    ),
                    filters: [],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "api_key",
                    ),
                    filters: [],
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_expr_with_number_variable() {
        let test_str = "1";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''1'' at 0..1 expected expr,
            ],
        }
        ",
        );
    }



    #[test]
    fn it_parses_expr_with_decode_filter() {
        let test_str = "api_key decode \"gb2312\"";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_expr_with_filters_missing_space_delimeter() {
        let test_str = r#"response jsonpath "$.names"count"#;
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''c'' at 27..28 expected spacing, or end of input,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_expr_with_days_after_now_filter() {
        let test_str = "expiration_date daysAfterNow";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "expiration_date",
                    ),
                    filters: [
                        DaysAfterNow,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "expiration_date",
                    ),
                    filters: [
                        DaysBeforeNow,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "document",
                    ),
                    filters: [
                        HtmlEscape,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "escaped_document",
                    ),
                    filters: [
                        HtmlUnescape,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "inflation_rate",
                    ),
                    filters: [
                        ToFloat,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "id",
                    ),
                    filters: [
                        ToInt,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "encoded_url",
                    ),
                    filters: [
                        UrlDecode,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "url",
                    ),
                    filters: [
                        UrlEncode,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
                Expr {
                    variable: VariableName(
                        "url",
                    ),
                    filters: [
                        UrlEncode,
                    ],
                },
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );

    }

}

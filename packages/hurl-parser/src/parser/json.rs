use chumsky::prelude::*;

use super::{
    method::method_line_parser,
    template::template_parser,
    types::{InterpolatedString, InterpolatedStringPart, Json, JsonKeyValue},
};

fn json_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    just('\\')
        .ignore_then(choice((
            just('\\'),
            just('/'),
            just('"'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('u').ignore_then(text::digits(16).exactly(4).to_slice().validate(
                |digits, e, emitter| {
                    char::from_u32(u32::from_str_radix(digits, 16).unwrap()).unwrap_or_else(|| {
                        emitter.emit(Rich::custom(e.span(), "invalid unicode character"));
                        '\u{FFFD}' // unicode replacement character
                    })
                },
            )),
        )))
        .labelled("json-string-escaped-char")
        .boxed()
}

fn json_string_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let json_string_content = choice((
        none_of(r#"\"{"#).labelled("json-string-text"),
        json_string_escaped_char_parser(),
        //opening curly brackes are valid as long as they are not followed by a
        //second curly bracket since two denote the start of a template
        //(this observation isn't explicit in the grammer but was determined from experiments
        just('{').then(just('{').not().rewind()).to('{'),
    ))
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("json-string-content");

    let json_string = choice((
        json_string_content,
        template_parser().map(InterpolatedStringPart::Template),
    ))
    .repeated()
    .collect::<Vec<InterpolatedStringPart>>()
    .map(|v| InterpolatedString { parts: v })
    .delimited_by(just('"'), just('"'))
    .labelled("json-string")
    .boxed();

    json_string
}

fn json_number_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone
{
    let digits = text::digits(10).to_slice();

    let frac = just('.').then(digits.clone());

    let exp = just('e')
        .or(just('E'))
        .then(one_of("+-").or_not())
        .then(digits.clone());

    let number = just('-')
        .or_not()
        .then(text::int(10))
        .then(frac.or_not())
        .then(exp.or_not())
        .to_slice()
        //TODO decide if I want to parse this into integers and floats.
        //I currently don't think it is needed for validation reasons. It might
        //be useful if I want to reduce memory footprint
        .map(|s: &str| s.to_string())
        .labelled("json-number")
        .boxed();

    number
}

fn json_object_parser<'a, T: Parser<'a, &'a str, Json, extra::Err<Rich<'a, char>>> + Clone>(
    json_value: T,
) -> impl Parser<'a, &'a str, Vec<JsonKeyValue>, extra::Err<Rich<'a, char>>> + Clone {
    let json_key_value = json_string_parser()
        .then_ignore(just(':').padded())
        .then(json_value)
        .map(|(key, value)| JsonKeyValue { key, value })
        .labelled("json-key-value");

    let json_object = json_key_value
        .separated_by(
            just(',').padded().recover_with(skip_then_retry_until(
                any().ignored(),
                one_of(",}")
                    .ignored()
                    .or(method_line_parser(true).ignored()),
            )),
        )
        .collect::<Vec<_>>()
        .padded()
        .delimited_by(
            just('{'),
            just('}')
                .ignored()
                .recover_with(via_parser(end()))
                .recover_with(skip_then_retry_until(
                    any().ignored(),
                    end().or(method_line_parser(true).ignored()),
                )),
        )
        .labelled("json-object");
    json_object
}

pub fn json_value_parser<'a>() -> impl Parser<'a, &'a str, Json, extra::Err<Rich<'a, char>>> + Clone
{
    let json_value = recursive(|json_value| {
        let array = json_value
            .clone()
            .separated_by(
                just(',').padded().recover_with(skip_then_retry_until(
                    any().ignored(),
                    one_of(",]")
                        .ignored()
                        .or(method_line_parser(true).ignored()),
                )),
            )
            .collect()
            .padded()
            .delimited_by(
                just('['),
                just(']')
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(
                        any().ignored(),
                        end().or(method_line_parser(true).ignored()),
                    )),
            )
            .boxed();

        let json_value_unpadded = choice((
            template_parser().map(Json::Template),
            just("null").to(Json::Null),
            just("true").to(Json::Bool(true)),
            just("false").to(Json::Bool(false)),
            json_number_parser().map(Json::Num),
            json_string_parser().map(Json::InterpolatedString),
            array.map(Json::Array),
            json_object_parser(json_value).map(Json::Object),
        ))
        .recover_with(via_parser(nested_delimiters(
            '{',
            '}',
            [('[', ']')],
            |_| Json::Invalid,
        )))
        .recover_with(via_parser(nested_delimiters(
            '[',
            ']',
            [('{', '}')],
            |_| Json::Invalid,
        )))
        .recover_with(skip_then_retry_until(
            any().ignored(),
            one_of(",]}")
                .ignored()
                .or(method_line_parser(true).ignored()),
        ));

        text::whitespace().ignore_then(json_value_unpadded)
    })
    .labelled("json-value")
    .boxed();

    json_value
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;
    use insta::assert_debug_snapshot;

    use crate::parser::json::json_value_parser;

    #[test]
    fn it_parses_template() {
        let test_str = r#"{{api-key}}"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Template(
                    Template {
                        expr: Expr {
                            variable: VariableName(
                                "api-key",
                            ),
                            filters: [],
                        },
                    },
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parse_empty_json_object() {
        let test_str = r#"{}"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Object(
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parse_empty_json_object_with_whitespace() {
        let test_str = r#"{    }"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Object(
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parse_empty_json_object_with_whitespace_and_newlines() {
        let test_str = r#"{  


    }"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Object(
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_empty_array() {
        let test_str = r#"[]"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Array(
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_empty_array_with_whitespace() {
        let test_str = r#"[    ]"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Array(
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_empty_array_with_whitespace_and_newlines() {
        let test_str = r#"[  


            ]"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Array(
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_string() {
        let test_str = r#""test""#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString(
                    InterpolatedString {
                        parts: [
                            Str(
                                "test",
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
    fn it_errors_empty_template_in_string() {
        //TODO try to improve error message
        let test_str = r#""{{ }}""#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''}'' at 4..5 expected spacing, or expr,
                found ''}'' at 5..6 expected end of input,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_templates_in_object_key_and_value_strings() {
        let test_str = r#"{
            "{{key}}": "{{value}}"
            }"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Object(
                    [
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "key",
                                                ),
                                                filters: [],
                                            },
                                        },
                                    ),
                                ],
                            },
                            value: InterpolatedString(
                                InterpolatedString {
                                    parts: [
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "value",
                                                    ),
                                                    filters: [],
                                                },
                                            },
                                        ),
                                    ],
                                },
                            ),
                        },
                    ],
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_templates_in_object_value() {
        let test_str = r#"{
            "pet_count": {{count}}
            }"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Object(
                    [
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "pet_count",
                                    ),
                                ],
                            },
                            value: Template(
                                Template {
                                    expr: Expr {
                                        variable: VariableName(
                                            "count",
                                        ),
                                        filters: [],
                                    },
                                },
                            ),
                        },
                    ],
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_integer() {
        let test_str = r#"55"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Num(
                    "55",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_float() {
        let test_str = r#"55.05"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Num(
                    "55.05",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_number_with_decimal_point_but_no_decimal_digits() {
        let test_str = r#"55."#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found end of input at 3..3 expected any,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_number_exponent() {
        let test_str = r#"55e+5"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Num(
                    "55e+5",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_full_number() {
        let test_str = r#"-55.05e+5"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Num(
                    "-55.05e+5",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_true() {
        let test_str = r#"true"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Bool(
                    true,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_false() {
        let test_str = r#"false"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Bool(
                    false,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_null() {
        let test_str = r#"null"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Null,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parse_complex_json_object() {
        let test_str = r#"{
            "type": 49,
            "id": "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
            "pet_types": [
                "cat",
                "dog",
                "hampster"
            ],
            "pets": [
                {
                    "type": "cat",
                    "mood": "annoyed"
                },
                {
                    "type": "dog",
                    "mood": "excited"
                },
                {
                    "type": "hampster",
                    "mood": "lazy"
                }
            ]
        }"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Object(
                    [
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "type",
                                    ),
                                ],
                            },
                            value: Num(
                                "49",
                            ),
                        },
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "id",
                                    ),
                                ],
                            },
                            value: InterpolatedString(
                                InterpolatedString {
                                    parts: [
                                        Str(
                                            "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
                                        ),
                                    ],
                                },
                            ),
                        },
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "pet_types",
                                    ),
                                ],
                            },
                            value: Array(
                                [
                                    InterpolatedString(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "cat",
                                                ),
                                            ],
                                        },
                                    ),
                                    InterpolatedString(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "dog",
                                                ),
                                            ],
                                        },
                                    ),
                                    InterpolatedString(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "hampster",
                                                ),
                                            ],
                                        },
                                    ),
                                ],
                            ),
                        },
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "pets",
                                    ),
                                ],
                            },
                            value: Array(
                                [
                                    Object(
                                        [
                                            JsonKeyValue {
                                                key: InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "type",
                                                        ),
                                                    ],
                                                },
                                                value: InterpolatedString(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "cat",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                            JsonKeyValue {
                                                key: InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "mood",
                                                        ),
                                                    ],
                                                },
                                                value: InterpolatedString(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "annoyed",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                        ],
                                    ),
                                    Object(
                                        [
                                            JsonKeyValue {
                                                key: InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "type",
                                                        ),
                                                    ],
                                                },
                                                value: InterpolatedString(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "dog",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                            JsonKeyValue {
                                                key: InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "mood",
                                                        ),
                                                    ],
                                                },
                                                value: InterpolatedString(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "excited",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                        ],
                                    ),
                                    Object(
                                        [
                                            JsonKeyValue {
                                                key: InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "type",
                                                        ),
                                                    ],
                                                },
                                                value: InterpolatedString(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "hampster",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                            JsonKeyValue {
                                                key: InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "mood",
                                                        ),
                                                    ],
                                                },
                                                value: InterpolatedString(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "lazy",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                        ],
                                    ),
                                ],
                            ),
                        },
                    ],
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_trailing_comma_in_object() {
        let test_str = r#"{
            "type": "cat",
            "mood": "annoyed",
        }"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Object(
                    [
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "type",
                                    ),
                                ],
                            },
                            value: InterpolatedString(
                                InterpolatedString {
                                    parts: [
                                        Str(
                                            "cat",
                                        ),
                                    ],
                                },
                            ),
                        },
                        JsonKeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "mood",
                                    ),
                                ],
                            },
                            value: InterpolatedString(
                                InterpolatedString {
                                    parts: [
                                        Str(
                                            "annoyed",
                                        ),
                                    ],
                                },
                            ),
                        },
                    ],
                ),
            ),
            errs: [
                found ''}'' at 68..69 expected json-key-value,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_extra_comma_in_array() {
        let test_str = r#"[
                "cat",
                "dog",
                "hampster",
        ]"#;
        assert_debug_snapshot!(
        json_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Array(
                    [
                        InterpolatedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "cat",
                                    ),
                                ],
                            },
                        ),
                        InterpolatedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "dog",
                                    ),
                                ],
                            },
                        ),
                        InterpolatedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "hampster",
                                    ),
                                ],
                            },
                        ),
                    ],
                ),
            ),
            errs: [
                found '']'' at 84..85 expected whitespace, template, ''n'', ''t'', ''f'', json-number, json-string, ''['', or json-object,
            ],
        }
        "#,
        );
    }

    // #[test]
    // fn it_errors_unclosed_object() {
    //     // let test_str = r#"{
    //     //     "type": 49,
    //     //     "id": "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
    //     //     "pet_types": [
    //     //         "cat",
    //     //         "dog",
    //     //         "hampster"
    //     //     ],
    //     //     "pets": [
    //     //         {
    //     //             "type": "cat",
    //     //             "mood": "annoyed"
    //     //         },
    //     //         {
    //     //             "type": "dog",
    //     //             "mood": "excited"
    //     //         },
    //     //         {
    //     //             "type": "hampster",
    //     //             "mood": "lazy"
    //     //         }
    //     //
    //     // }"#;
    //     //
    //
    //     let test_str = r#"{
    //         "type": 49,
    //         "id": "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
    //         "pet_types": [
    //             "cat",
    //             "dog",
    //             "hampster"
    //         ],
    //         "pets": [
    //             {
    //                 "type": "cat",
    //                 "mood": "annoyed"
    //             },
    //             {
    //                 "type": "dog",
    //                 "mood": "excited"
    //             },
    //             {
    //                 "type": "hampster",
    //                 "mood": "lazy"
    //             }
    //
    //     }"#;
    //     assert_debug_snapshot!(
    //     json_value_parser().parse(test_str),
    //         @r"
    //     ParseResult {
    //         output: None,
    //         errs: [
    //             found ''}'' at 573..574 expected '','', or '']'',
    //             found ''{'' at 347..348 expected json-key-value,
    //             found '','' at 439..440 expected end of input,
    //         ],
    //     }
    //     ",
    //     );
    // }
}

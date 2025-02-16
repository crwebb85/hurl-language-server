use super::{
    multiline_string::multiline_string_parser,
    oneline_base64::oneline_base64_parser,
    oneline_file::oneline_file_parser,
    oneline_hex::oneline_hex_parser,
    oneline_string::oneline_string_parser,
    primitives::sp_parser,
    quoted_string::quoted_string_parser,
    template::template_parser,
    types::{Predicate, PredicateFunc, PredicatePrefixOperator, PredicateValue},
};
use chumsky::prelude::*;
use ordered_float::OrderedFloat;

pub fn predicate_parser<'a>(
) -> impl Parser<'a, &'a str, Predicate, extra::Err<Rich<'a, char>>> + Clone {
    let fraction = just('.').then(text::digits(10));
    let number = just("-")
        .or_not()
        .then(text::int(10))
        .then(fraction.or_not())
        .to_slice()
        .validate(|number: &str, e, emitter| {
            if number.contains('.') {
                match number.parse::<f64>() {
                    Ok(n) => PredicateValue::Float(OrderedFloat::<f64>::from(n)),
                    Err(_) => {
                        emitter.emit(Rich::custom(e.span(), "invalid float"));
                        PredicateValue::Invalid
                    }
                }
            } else {
                match number.parse::<i64>() {
                    Ok(n) => PredicateValue::Integer(n),
                    Err(_) => PredicateValue::BigInteger(number.to_string()),
                }
            }
        });

    let predicate_value = choice((
        just("true").to(PredicateValue::Boolean(true)),
        just("false").to(PredicateValue::Boolean(false)),
        just("null").to(PredicateValue::Null),
        number,
        template_parser().map(PredicateValue::Template),
        quoted_string_parser().map(PredicateValue::QuotedString),
        oneline_hex_parser().map(PredicateValue::OneLineHex),
        oneline_file_parser().map(PredicateValue::OneLineFile),
        oneline_base64_parser().map(PredicateValue::OneLineBase64),
        oneline_string_parser().map(PredicateValue::OneLineString),
        multiline_string_parser().map(PredicateValue::MultilineString),
    ));

    let predicate_function = choice((
        just("==")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .map(|(_, value)| PredicateFunc::Equal { value }),
        just("!=")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .map(|(_, value)| PredicateFunc::NotEqual { value }),
        just(">")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::Integer(_)
                | PredicateValue::Float(_)
                | PredicateValue::BigInteger(_)
                | PredicateValue::Template(_)//Off spec but is in official parser (called
                                             //placeholder expression in official parser)
                | PredicateValue::OneLineString(_)//Off spec but is in official parser
                | PredicateValue::QuotedString(_) => PredicateFunc::Greater { value },

                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::Greater { value }
                }
            }),
        just(">=")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::Integer(_)
                | PredicateValue::Float(_)
                | PredicateValue::BigInteger(_)
                | PredicateValue::Template(_)//Off spec but is in official parser (called
                                             //placeholder expression in official parser)
                | PredicateValue::OneLineString(_)//Off spec but is in official parser
                | PredicateValue::QuotedString(_) => PredicateFunc::GreaterOrEqual { value },
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::GreaterOrEqual { value }
                }
            }),
        just("<")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::Integer(_)
                | PredicateValue::Float(_)
                | PredicateValue::BigInteger(_)
                | PredicateValue::Template(_)//Off spec but is in official parser (called
                                             //placeholder expression in official parser)
                | PredicateValue::OneLineString(_)//Off spec but is in official parser
                | PredicateValue::QuotedString(_) => PredicateFunc::Less { value },
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::Less { value }
                }
            }),
        just("<=")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::Integer(_)
                | PredicateValue::Float(_)
                | PredicateValue::BigInteger(_)
                | PredicateValue::Template(_)//Off spec but is in official parser (called
                                             //placeholder expression in official parser)
                | PredicateValue::OneLineString(_)//Off spec but is in official parser
                | PredicateValue::QuotedString(_) => PredicateFunc::LessOrEqual { value },
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::LessOrEqual { value }
                }
            }),
        just("startsWith")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::OneLineString(_)//Off spec but is in official parser
                | PredicateValue::QuotedString(_)
                | PredicateValue::OneLineHex(_)
                | PredicateValue::OneLineBase64(_) => PredicateFunc::StartWith { value },
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::StartWith { value }
                }
            }),
        just("endsWith")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::OneLineString(_)//Off spec but is in official parser
                | PredicateValue::QuotedString(_)
                | PredicateValue::OneLineHex(_)
                | PredicateValue::OneLineBase64(_) => PredicateFunc::EndWith { value },
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::EndWith { value }
                }
            }),
        just("contains")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                //TODO the spec says on quoted strings are allowed but I don't see where in the 
                //code it restricts the types for this predicate function's value
                PredicateValue::OneLineString(_)//Off spec but but determined testing showed that
                                                //this works
                | PredicateValue::QuotedString(_) 
                | PredicateValue::OneLineHex(_)//Offspec but testing shows this workds
                | PredicateValue::OneLineBase64(_) // Offspec but testing shows this works
                => PredicateFunc::Contain { value },
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::Contain { value }
                }
            }),
        just("includes")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .map(|(_, value)| PredicateFunc::Include { value }),
        just("matches")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(predicate_value.clone())
            .validate(|(_, value): (_, PredicateValue), e, emitter| match value {
                PredicateValue::OneLineString(_)//Off spec but but determined testing showed that
                                                //this workds
                |PredicateValue::QuotedString(_) | PredicateValue::Regex(_) => {
                    PredicateFunc::Match { value }
                }
                _ => {
                    emitter.emit(Rich::custom(e.span(), "Unexpected predicate value"));
                    PredicateFunc::Match { value }
                }
            }),
        just("exists").to(PredicateFunc::Exists),
        just("isEmpty").to(PredicateFunc::IsEmpty),
        just("isInteger").to(PredicateFunc::IsInteger),
        just("isFloat").to(PredicateFunc::IsFloat),
        just("isBoolean").to(PredicateFunc::IsBoolean),
        just("isString").to(PredicateFunc::IsString),
        just("isCollection").to(PredicateFunc::IsCollection),
        just("isDate").to(PredicateFunc::IsDate),
        just("isIsoDate").to(PredicateFunc::IsIsoDate),
    ));
    let not_keyword = just("not")
        .to(PredicatePrefixOperator::Not)
        .then_ignore(sp_parser().repeated().at_least(1))
        .boxed();
    let predicate = not_keyword
        .or_not()
        .then(predicate_function)
        .map(|(prefix, function)| Predicate { prefix, function });
    predicate
}

#[cfg(test)]
mod predicate_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_equal_predicate_predicate() {
        let test_str = r#"== true"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Equal {
                        value: Boolean(
                            true,
                        ),
                    },
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_not_equal_predicate_predicate() {
        let test_str = r#"!= "dog""#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: NotEqual {
                        value: QuotedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "dog",
                                    ),
                                ],
                            },
                        ),
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_greater_predicate_predicate() {
        let test_str = r#"> 5.05"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Greater {
                        value: Float(
                            5.05,
                        ),
                    },
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_greater_or_equal_predicate_predicate() {
        let test_str = r#">= 0"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: GreaterOrEqual {
                        value: Integer(
                            0,
                        ),
                    },
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_less_predicate_predicate() {
        let test_str = r#"< 9"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Less {
                        value: Integer(
                            9,
                        ),
                    },
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_less_or_equal_predicate_predicate() {
        let test_str = r#"<= 20"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: LessOrEqual {
                        value: Integer(
                            20,
                        ),
                    },
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_start_with_predicate_predicate() {
        let test_str = r#"startsWith "abc""#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: StartWith {
                        value: QuotedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "abc",
                                    ),
                                ],
                            },
                        ),
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_end_with_predicate_predicate() {
        let test_str = r#"endsWith "xyz""#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: EndWith {
                        value: QuotedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "xyz",
                                    ),
                                ],
                            },
                        ),
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_contain_predicate_predicate() {
        let test_str = r#"contains "bird""#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Contain {
                        value: QuotedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "bird",
                                    ),
                                ],
                            },
                        ),
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_match_predicate_predicate() {
        let test_str = r#"matches "cat""#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Match {
                        value: QuotedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "cat",
                                    ),
                                ],
                            },
                        ),
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_include_predicate_predicate() {
        let test_str = r#"includes 5"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Include {
                        value: Integer(
                            5,
                        ),
                    },
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_exists_predicate() {
        let test_str = r#"exists"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: Exists,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_exists_predicate_inverted() {
        let test_str = r#"not  exists"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: Some(
                        Not,
                    ),
                    function: Exists,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_empty_predicate() {
        let test_str = r#"isEmpty"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsEmpty,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_integer_predicate() {
        let test_str = r#"isInteger"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsInteger,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_float_predicate() {
        let test_str = r#"isFloat"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsFloat,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_boolean_predicate() {
        let test_str = r#"isBoolean"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsBoolean,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_string_predicate() {
        let test_str = r#"isString"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsString,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_collection_predicate() {
        let test_str = r#"isCollection"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsCollection,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_date_predicate() {
        let test_str = r#"isDate"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsDate,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_is_iso_date_predicate() {
        let test_str = r#"isIsoDate"#;
        assert_debug_snapshot!(
        predicate_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Predicate {
                    prefix: None,
                    function: IsIsoDate,
                },
            ),
            errs: [],
        }
        ",
        );
    }
}

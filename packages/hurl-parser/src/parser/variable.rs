use super::{key_value::key_parser, quoted_string::quoted_string_parser, types::VariableValue};
use chumsky::prelude::*;
use ordered_float::OrderedFloat;

pub fn variable_value_parser<'a>(
) -> impl Parser<'a, &'a str, VariableValue, extra::Err<Rich<'a, char>>> + Clone {
    let fraction = just('.').then(text::digits(10));
    let number = just("-")
        .or_not()
        .then(text::int(10))
        .then(fraction.or_not())
        .to_slice()
        .validate(|number: &str, e, emitter| {
            if number.contains('.') {
                match number.parse::<f64>() {
                    Ok(n) => VariableValue::Float(OrderedFloat::<f64>::from(n)),
                    Err(_) => {
                        emitter.emit(Rich::custom(e.span(), "invalid float"));
                        VariableValue::Invalid
                    }
                }
            } else {
                match number.parse::<i64>() {
                    Ok(n) => VariableValue::Integer(n),
                    Err(_) => VariableValue::BigInteger(number.to_string()),
                }
            }
        });

    let variable_value = choice((
        just("null").to(VariableValue::Null),
        just("true").to(VariableValue::Boolean(true)),
        just("false").to(VariableValue::Boolean(false)),
        just("false").to(VariableValue::Boolean(false)),
        number,
        key_parser().map(VariableValue::String), //TODO the official grammer is wrong and what is
        //actually used is more similar to value_parser
        quoted_string_parser().map(VariableValue::String),
    ));
    variable_value.boxed()
}

//TODO add code action to wrap an invalid unquoted string in quotes
//TODO add code action to convert the invalid variable definition
// variable: bad_float=6.{{fraction}}
//
//to
//
// variable: good_float_string="6.{{fraction}}"
// variable: float_string={{good_float_string toFloat}}
//
// TODO add a lazy type evaluation to variables to validate if filters and templates are valid

#[cfg(test)]
mod option_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_null() {
        let test_str = r#"null"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
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
    fn it_parses_true() {
        let test_str = r#"true"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Boolean(
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
        variable_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Boolean(
                    false,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_unquoted_variable_string() {
        let test_str = "example.net";
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                String(
                    InterpolatedString {
                        parts: [
                            Str(
                                "example.net",
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
    fn it_parses_quoted_variable_string() {
        let test_str = r#""example.net""#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                String(
                    InterpolatedString {
                        parts: [
                            Str(
                                "example.net",
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
    fn it_parses_positive_integer() {
        let test_str = r#"420"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Integer(
                    420,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_negative_integer() {
        let test_str = r#"-420"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Integer(
                    -420,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_big_integer() {
        let test_str = r#"18446744073709551616"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                BigInteger(
                    "18446744073709551616",
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_positive_float() {
        let test_str = r#"420.69"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Float(
                    420.69,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_negative_float() {
        let test_str = r#"-420.69"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Float(
                    -420.69,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[ignore = "official grammer needs to be fixed for unquoted strings"]
    #[test]
    fn it_parses_integer_like_string_with_spaces_between_digits() {
        let test_str = r#"5 5"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"

        "#,
        );
    }

    #[ignore = "official grammer needs to be fixed for unquoted strings"]
    #[test]
    fn it_parses_integer_like_string_with_spaces_after_minus_sign() {
        let test_str = r#"-  5"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"

        "#,
        );
    }

    #[ignore = "official grammer needs to be fixed for unquoted strings"]
    #[test]
    fn it_parses_float_like_string_with_spaces_after_minus_sign() {
        let test_str = r#"-  5.5"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"

        "#,
        );
    }

    #[ignore = "official grammer needs to be fixed for unquoted strings"]
    #[test]
    fn it_parses_float_like_string_with_space_before_decimal_point() {
        let test_str = r#"-5 .5"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"

        "#,
        );
    }

    #[ignore = "official grammer needs to be fixed for unquoted strings"]
    #[test]
    fn it_parses_float_like_string_with_space_after_decimal_point() {
        let test_str = r#"-5. 5"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"

        "#,
        );
    }

    #[ignore = "official grammer needs to be fixed for unquoted strings"]
    #[test]
    fn it_parses_unquoted_string_with_non_alphanumeric_characters() {
        let test_str = r#"5*5"#;
        assert_debug_snapshot!(
        variable_value_parser().parse(test_str),
            @r#"

        "#,
        );
    }
}

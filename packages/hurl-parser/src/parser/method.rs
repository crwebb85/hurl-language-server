use chumsky::prelude::*;

use super::{
    key_value::value_parser,
    primitives::{ascii_alphabetic_uppercase_parser, lt_parser, sp_parser},
    types::{Method, Url},
};

fn method_parser<'a>(
    strict: bool,
) -> impl Parser<'a, &'a str, Method, extra::Err<Rich<'a, char>>> + Clone {
    if strict {
        ascii_alphabetic_uppercase_parser()
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(|method| Method { value: method })
            .boxed()
    } else {
        text::ident()
            .to_slice()
            .validate(|ident: &str, e, emitter| {
                match ident.find(|c| !char::is_ascii_uppercase(&c)) {
                    Some(index) => emitter.emit(Rich::custom(
                        e.span(),
                        format!(
                            "Invalid character '{}'. Method must be ascii uppercase.",
                            ident.chars().nth(index).unwrap() // We know the character is at the index
                        ),
                    )),
                    None => (),
                };

                Method {
                    value: ident.to_string(),
                }
            })
            .boxed()
    }
}

pub fn method_line_parser<'a>(
    strict: bool,
) -> impl Parser<'a, &'a str, (Method, Url), extra::Err<Rich<'a, char>>> + Clone {
    if strict {
        let method_line = sp_parser()
            .repeated()
            .ignore_then(method_parser(strict))
            .padded_by(sp_parser().repeated().at_least(1))
            .then(value_parser().map(Url::Url))
            .then_ignore(lt_parser());
        method_line.boxed()
    } else {
        let method_line = sp_parser()
            .repeated()
            .ignore_then(method_parser(strict))
            .then(
                sp_parser()
                    .repeated()
                    .at_least(1)
                    .ignore_then(value_parser().or_not())
                    .or_not()
                    .validate(|spaces_and_url, e, emitter| match spaces_and_url {
                        Some(url) => match url {
                            Some(u) => Url::Url(u),
                            None => {
                                emitter.emit(Rich::custom(e.span(), "missing url"));
                                Url::Missing
                            }
                        },
                        None => {
                            emitter.emit(Rich::custom(e.span(), "missing url"));
                            Url::Missing
                        }
                    }),
            )
            .then_ignore(lt_parser());
        method_line.boxed()
    }
}

#[cfg(test)]
mod method_line_tests {
    use super::*;
    use insta::assert_debug_snapshot;
    //TODO add tests

    //TODO error handle if missing URL

    #[test]
    fn it_recovers_missing_space_and_missing_url() {
        let test_str = "GET";
        assert_debug_snapshot!(
        method_line_parser(false).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    Method {
                        value: "GET",
                    },
                    Missing,
                ),
            ),
            errs: [
                missing url at 3..3,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_recovers_invalid_casing_method_for_non_strict_parsing() {
        let test_str = "GeT https://example.org";
        assert_debug_snapshot!(
        method_line_parser(false).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    Method {
                        value: "GeT",
                    },
                    Url(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
                        },
                    ),
                ),
            ),
            errs: [
                Invalid character 'e'. Method must be ascii uppercase. at 0..3,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_recovers_missing_url() {
        let test_str = "GET ";
        assert_debug_snapshot!(
        method_line_parser(false).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    Method {
                        value: "GET",
                    },
                    Missing,
                ),
            ),
            errs: [
                missing url at 3..4,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_parse_get_with_url() {
        let test_str = "GET https://example.org/";
        assert_debug_snapshot!(
        method_line_parser(false).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    Method {
                        value: "GET",
                    },
                    Url(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/",
                                ),
                            ],
                        },
                    ),
                ),
            ),
            errs: [],
        }
        "#,
        );
    }
}

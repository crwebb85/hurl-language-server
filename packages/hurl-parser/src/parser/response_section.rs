use super::{
    expr::filters_parser,
    key_value::key_parser,
    predicate::predicate_parser,
    primitives::{lt_parser, sp_parser},
    query::query_parser,
    quoted_string::quoted_string_parser,
    types::{Assert, AssertsSection, Capture, CapturesSection, ResponseSection},
};
use chumsky::prelude::*;

pub fn response_sections_parser<'a>(
) -> impl Parser<'a, &'a str, Vec<ResponseSection>, extra::Err<Rich<'a, char>>> + Clone {
    let capture_line = key_parser()
        .padded_by(sp_parser().repeated())
        .then_ignore(just(':').padded_by(sp_parser().repeated()))
        .then(query_parser())
        .then_ignore(sp_parser().repeated())
        .then(filters_parser(quoted_string_parser()))
        .then_ignore(lt_parser())
        .map(|((key, query), filters)| Capture {
            key,
            query,
            filters,
        });

    let captures = capture_line.repeated().collect::<Vec<Capture>>();

    let assert_line = query_parser()
        .delimited_by(sp_parser().repeated(), sp_parser().repeated().at_least(1))
        .then(filters_parser(quoted_string_parser()))
        .then_ignore(sp_parser().repeated())
        .then(predicate_parser())
        .then_ignore(lt_parser())
        .map(|((query, filters), predicate)| Assert {
            query,
            filters,
            predicate,
        });
    let asserts = assert_line.repeated().collect::<Vec<Assert>>();

    let captures_section = just("[Captures]")
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(captures)
        .map(|(_, captures)| ResponseSection::CapturesSection(CapturesSection { captures }));

    let asserts_section = just("[Asserts]")
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(asserts)
        .map(|(_, asserts)| ResponseSection::AssertsSection(AssertsSection { asserts }));

    let response_sections = choice((captures_section, asserts_section))
        .repeated()
        .collect::<Vec<ResponseSection>>()
        .boxed();

    //TODO sections can only be defined once per entry's response section. So you can't have [Asserts] defined
    //twice and so should be a diagnostic error.
    response_sections
}

#[cfg(test)]
mod response_section_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_captures_section() {
        let test_str = r#"[Captures]
        csrf_token: xpath "string(//meta[@name='_csrf_token']/@content)"
        next_url: header "Location"
        "#;
        assert_debug_snapshot!(
        response_sections_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
                    CapturesSection(
                        CapturesSection {
                            captures: [
                                Capture {
                                    key: InterpolatedString {
                                        parts: [
                                            Str(
                                                "csrf_token",
                                            ),
                                        ],
                                    },
                                    query: Xpath(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "string(//meta[@name='_csrf_token']/@content)",
                                                ),
                                            ],
                                        },
                                    ),
                                    filters: [],
                                },
                                Capture {
                                    key: InterpolatedString {
                                        parts: [
                                            Str(
                                                "next_url",
                                            ),
                                        ],
                                    },
                                    query: Header(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "Location",
                                                ),
                                            ],
                                        },
                                    ),
                                    filters: [],
                                },
                            ],
                        },
                    ),
                ],
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_asserts_section() {
        let test_str = r#"[Asserts]
            header "Content-Type" == "text/html; charset=utf8"
            # contains the value in hex
            bytes contains hex,5468697320697320616E206578616D706C65;
            # contains the string value
            body contains "This is an example"        
        "#;
        assert_debug_snapshot!(
        response_sections_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
                    AssertsSection(
                        AssertsSection {
                            asserts: [
                                Assert {
                                    query: Header(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "Content-Type",
                                                ),
                                            ],
                                        },
                                    ),
                                    filters: [],
                                    predicate: Predicate {
                                        prefix: None,
                                        function: Equal {
                                            value: QuotedString(
                                                InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "text/html; charset=utf8",
                                                        ),
                                                    ],
                                                },
                                            ),
                                        },
                                    },
                                },
                                Assert {
                                    query: Bytes,
                                    filters: [],
                                    predicate: Predicate {
                                        prefix: None,
                                        function: Contain {
                                            value: OneLineHex(
                                                "5468697320697320616E206578616D706C65",
                                            ),
                                        },
                                    },
                                },
                                Assert {
                                    query: Body,
                                    filters: [],
                                    predicate: Predicate {
                                        prefix: None,
                                        function: Contain {
                                            value: QuotedString(
                                                InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "This is an example",
                                                        ),
                                                    ],
                                                },
                                            ),
                                        },
                                    },
                                },
                            ],
                        },
                    ),
                ],
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_captures_and_assert_section() {
        let test_str = r#"[Captures]
            csrf_token: xpath "string(//meta[@name='_csrf_token']/@content)"
            next_url: header "Location"
            [Asserts]
            header "Content-Type" == "text/html; charset=utf8"
            # contains the value in hex
            bytes contains hex,5468697320697320616E206578616D706C65;
            # contains the string value
            body contains "This is an example"
        "#;
        assert_debug_snapshot!(
        response_sections_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
                    CapturesSection(
                        CapturesSection {
                            captures: [
                                Capture {
                                    key: InterpolatedString {
                                        parts: [
                                            Str(
                                                "csrf_token",
                                            ),
                                        ],
                                    },
                                    query: Xpath(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "string(//meta[@name='_csrf_token']/@content)",
                                                ),
                                            ],
                                        },
                                    ),
                                    filters: [],
                                },
                                Capture {
                                    key: InterpolatedString {
                                        parts: [
                                            Str(
                                                "next_url",
                                            ),
                                        ],
                                    },
                                    query: Header(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "Location",
                                                ),
                                            ],
                                        },
                                    ),
                                    filters: [],
                                },
                            ],
                        },
                    ),
                    AssertsSection(
                        AssertsSection {
                            asserts: [
                                Assert {
                                    query: Header(
                                        InterpolatedString {
                                            parts: [
                                                Str(
                                                    "Content-Type",
                                                ),
                                            ],
                                        },
                                    ),
                                    filters: [],
                                    predicate: Predicate {
                                        prefix: None,
                                        function: Equal {
                                            value: QuotedString(
                                                InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "text/html; charset=utf8",
                                                        ),
                                                    ],
                                                },
                                            ),
                                        },
                                    },
                                },
                                Assert {
                                    query: Bytes,
                                    filters: [],
                                    predicate: Predicate {
                                        prefix: None,
                                        function: Contain {
                                            value: OneLineHex(
                                                "5468697320697320616E206578616D706C65",
                                            ),
                                        },
                                    },
                                },
                                Assert {
                                    query: Body,
                                    filters: [],
                                    predicate: Predicate {
                                        prefix: None,
                                        function: Contain {
                                            value: QuotedString(
                                                InterpolatedString {
                                                    parts: [
                                                        Str(
                                                            "This is an example",
                                                        ),
                                                    ],
                                                },
                                            ),
                                        },
                                    },
                                },
                            ],
                        },
                    ),
                ],
            ),
            errs: [],
        }
        "#,
        );
    }
}

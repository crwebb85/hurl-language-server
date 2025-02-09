use super::body::body_parser;
use super::header::headers_parser;
use super::http_status::http_status_line_parser;
use super::response_section::response_sections_parser;
use super::types::Response;
use chumsky::prelude::*;

pub fn response_parser<'a>(
) -> impl Parser<'a, &'a str, Response, extra::Err<Rich<'a, char>>> + Clone {
    http_status_line_parser()
        .then(headers_parser())
        .then(response_sections_parser())
        .then(body_parser().or_not())
        .map(
            |((((version, status), headers), response_sections), body)| Response {
                version,
                status,
                headers,
                response_sections,
                body,
            },
        )
        .labelled("response")
}

#[cfg(test)]
mod http_status_line_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_http_200() {
        let test_str = "HTTP 200";
        assert_debug_snapshot!(
        response_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Response {
                    version: Http,
                    status: Code(
                        200,
                    ),
                    headers: [],
                    response_sections: [],
                    body: None,
                },
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_http3_200_with_json_body() {
        let test_str = r#"HTTP/3 200
        {
            "pets": ["cat", "dog"]
        }
            "#;
        assert_debug_snapshot!(
        response_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Response {
                    version: Http3,
                    status: Code(
                        200,
                    ),
                    headers: [],
                    response_sections: [],
                    body: Some(
                        Body {
                            bytes: JsonValue(
                                Object(
                                    [
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
                                                ],
                                            ),
                                        },
                                    ],
                                ),
                            ),
                        },
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_http3_200_with_headers_and_json_body() {
        let test_str = r#"HTTP/3 200
        Access-Control-Allow-Origin: *
        Connection: Keep-Alive
        Content-Encoding: gzip
        Content-Type: text/html; charset=utf-8
        Server: Apache
        Transfer-Encoding: chunked
        {
            "pets": ["cat", "dog"]
        }
            "#;
        assert_debug_snapshot!(
        response_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Response {
                    version: Http3,
                    status: Code(
                        200,
                    ),
                    headers: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "Access-Control-Allow-Origin",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "*",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "Connection",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "Keep-Alive",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "Content-Encoding",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "gzip",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "Content-Type",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "text/html; charset=utf-8",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "Server",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "Apache",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "Transfer-Encoding",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "chunked",
                                    ),
                                ],
                            },
                        },
                    ],
                    response_sections: [],
                    body: Some(
                        Body {
                            bytes: JsonValue(
                                Object(
                                    [
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
                                                ],
                                            ),
                                        },
                                    ],
                                ),
                            ),
                        },
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }
}

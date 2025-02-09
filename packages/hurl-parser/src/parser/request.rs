use super::body::body_parser;
use super::header::headers_parser;
use super::method::method_line_parser;
use super::request_section::request_sections_parser;
use super::types::Request;
use chumsky::prelude::*;

pub fn request_parser<'a>() -> impl Parser<'a, &'a str, Request, extra::Err<Rich<'a, char>>> + Clone
{
    let request = method_line_parser(false)
        .then(headers_parser())
        .then(request_sections_parser())
        .then(body_parser().or_not())
        .map(
            |((((method_value, url_value_string), headers), request_sections), body)| Request {
                method: method_value,
                url: url_value_string,
                headers,
                request_sections,
                body,
            },
        )
        .labelled("request");
    request.boxed()
}

//TODO add tests

#[cfg(test)]
mod request_parser_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[ignore = "fix parser to handle this edge case"]
    #[test]
    fn it_parses_http_200_with_warning() {
        //TODO
        let test_str = "HTTP 200";
        assert_debug_snapshot!(
        request_parser().parse(test_str),
            @r#"
        "#,
        );
    }

    #[test]
    fn it_recovers_missing_space_and_missing_url() {
        let test_str = "GET";
        assert_debug_snapshot!(
        request_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Request {
                    method: Method {
                        value: "GET",
                    },
                    url: Missing,
                    headers: [],
                    request_sections: [],
                    body: None,
                },
            ),
            errs: [
                missing url at 3..3,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_recovers_missing_url() {
        let test_str = "GET ";
        assert_debug_snapshot!(
        request_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Request {
                    method: Method {
                        value: "GET",
                    },
                    url: Missing,
                    headers: [],
                    request_sections: [],
                    body: None,
                },
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
        request_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Request {
                    method: Method {
                        value: "GET",
                    },
                    url: Url(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/",
                                ),
                            ],
                        },
                    ),
                    headers: [],
                    request_sections: [],
                    body: None,
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_recovers_invalid_casing_method_for_non_strict_parsing() {
        let test_str = "GeT https://example.org";
        assert_debug_snapshot!(
        request_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Request {
                    method: Method {
                        value: "GeT",
                    },
                    url: Url(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
                        },
                    ),
                    headers: [],
                    request_sections: [],
                    body: None,
                },
            ),
            errs: [
                Invalid character 'e'. Method must be ascii uppercase. at 0..3,
            ],
        }
        "#,
        );
    }
}

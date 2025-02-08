use chumsky::prelude::*;

use super::{
    json::json_value_parser,
    multiline_string::multiline_string_parser,
    oneline_base64::oneline_base64_parser,
    oneline_file::oneline_file_parser,
    oneline_hex::oneline_hex_parser,
    oneline_string::oneline_string_parser,
    primitives::lt_parser,
    types::{Body, Bytes},
};

//TODO it may be easier to do error recovery if I select all the text until the
// lt_parser().then(choice((
//  request_method_line_parser(false),
//  http_version_status_line_parser(),
//  end()
// )))
//
// and then parse that section of text into the body

pub fn bytes_parser<'a>() -> impl Parser<'a, &'a str, Bytes, extra::Err<Rich<'a, char>>> + Clone {
    choice((
        //xml_parser().map(Bytes::Xml), //TODO when hurl implements syntax for xml bytes
        multiline_string_parser().map(Bytes::MultilineString),
        oneline_string_parser().map(Bytes::OneLineString),
        oneline_base64_parser().map(Bytes::OneLineBase64),
        oneline_file_parser().map(Bytes::OneLineFile),
        oneline_hex_parser().map(Bytes::OneLineHex),
        json_value_parser().map(Bytes::JsonValue),
    ))
    .labelled("bytes")
    .boxed()
}

pub fn body_parser<'a>() -> impl Parser<'a, &'a str, Body, extra::Err<Rich<'a, char>>> + Clone {
    bytes_parser()
        .then_ignore(lt_parser())
        .map(|bytes| Body { bytes })
        .labelled("body")
        .boxed()
}

#[cfg(test)]
mod body_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_hex_string_body() {
        let test_str = r#"hex, 2AFA;
"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: OneLineHex(
                        "2AFA",
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_base64_string_body() {
        let test_str = r#"base64, VGhpcyBpcyBhIHRlc3Q=;"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: OneLineBase64(
                        "VGhpcyBpcyBhIHRlc3Q=",
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_string_body() {
        let test_str = r#"`test`"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: OneLineString(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "test",
                                ),
                            ],
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
    fn it_parses_multiline_string_body() {
        let test_str = r#"```
    this is some text
    another line
    hello world
                ```"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: MultilineString(
                        MultilineString {
                            type: None,
                            attributes: [],
                            content: InterpolatedString {
                                parts: [
                                    Str(
                                        "    this is some text\n    another line\n    hello world\n                ",
                                    ),
                                ],
                            },
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
    fn it_parses_oneline_file_body() {
        let test_str = "file,example.txt;";
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: OneLineFile(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "example.txt",
                                ),
                            ],
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
    fn it_parses_json_object_body() {
        let test_str = r#"{
            "pet_count": 3
            }"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: JsonValue(
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
                                    value: Num(
                                        "3",
                                    ),
                                },
                            ],
                        ),
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_json_array_body() {
        let test_str = r#"[
            1, 2, 3
            ]"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: JsonValue(
                        Array(
                            [
                                Num(
                                    "1",
                                ),
                                Num(
                                    "2",
                                ),
                                Num(
                                    "3",
                                ),
                            ],
                        ),
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_json_number_body() {
        let test_str = r#"12345"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: JsonValue(
                        Num(
                            "12345",
                        ),
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_json_string_body() {
        let test_str = r#""testing""#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: JsonValue(
                        InterpolatedString(
                            InterpolatedString {
                                parts: [
                                    Str(
                                        "testing",
                                    ),
                                ],
                            },
                        ),
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_json_template_body() {
        let test_str = r#"{{body}}"#;
        assert_debug_snapshot!(
        body_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Body {
                    bytes: JsonValue(
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "body",
                                    ),
                                    filters: [],
                                },
                            },
                        ),
                    ),
                },
            ),
            errs: [],
        }
        "#,
        );
    }
}

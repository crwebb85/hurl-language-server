#[cfg(test)]
mod tests {
    use crate::parser::ast_parser;
    use chumsky::Parser;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parse_simple_get() {
        let test_str = "GET https://example.org";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_simple_get_with_trailing_newline() {
        let test_str = "GET https://example.org\n";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_simple_get_with_newline_after_method() {
        let test_str = "GET\nhttps://example.org";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_simple_get_with_newline_after_method_and_after_url() {
        let test_str = "GET\nhttps://example.org\n";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_simple_get_with_extra_whitespace() {
        let test_str = "GET\n https://example.org";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_simple_post() {
        let test_str = "POST https://example.org";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "POST",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }
    #[test]
    fn it_parses_unknown_method() {
        let test_str = "FOO https://example.org";
        assert_debug_snapshot!(ast_parser().parse(test_str),@r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "FOO",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "# );
    }

    #[test]
    fn it_parses_header() {
        // let test_str = r#"
        // GET https://example.org/protected
        // Authorization: Basic Ym9iOnNlY3JldA==
        // "#;
        let test_str = "GET https://example.org/protected\nAuthorization: Basic Ym9iOnNlY3JldA==";

        //TODO parser needs to improve parsing of key-string and value-string to exclude the
        //as the leading space before "Basic" probably shouldn't be there.
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org/protected",
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "Authorization",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            " Basic Ym9iOnNlY3JldA==",
                                        ),
                                    ],
                                },
                            },
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parse_simple_get_with_leading_whitespace() {
        let test_str = "    GET https://example.org";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parse_multiple_entries_with_leading_whitespace() {
        //    GET https://example.org
        //GET https://example.org/protected
        //Authorization: Basic Ym9iOnNlY3JldA==";
        let test_str = "    GET https://example.org\nGET https://example.org/protected\nAuthorization: Basic Ym9iOnNlY3JldA==";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org/protected",
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "Authorization",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            " Basic Ym9iOnNlY3JldA==",
                                        ),
                                    ],
                                },
                            },
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parse_colon_in_header_value() {
        let test_str = "GET https://example.org\nkey: this:value:has:colons";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "key",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            " this:value:has:colons",
                                        ),
                                    ],
                                },
                            },
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parse_escaped_colon_in_header_key() {
        let test_str =
            "GET https://example.org\nkey\\:has\\:escaped\\:colons: thekeyhadescapedcolons";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "key:has:escaped:colons",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            " thekeyhadescapedcolons",
                                        ),
                                    ],
                                },
                            },
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parse_escaped_backslash_in_header_value() {
        let test_str = "GET https://example.org\nkey: thekeyhasescaped\\\\backslash";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "key",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            " thekeyhasescaped\\backslash",
                                        ),
                                    ],
                                },
                            },
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_error_from_unescaped_backslash_in_header_value() {
        //TODO improve error message and recovery from this type of error
        let test_str = "GET https://example.org\nkey: this\\valuehasanunescapedbackslash";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r"
        Err(
            [
                Simple {
                    span: 34..35,
                    reason: Unexpected,
                    expected: {
                        Some(
                            'r',
                        ),
                        Some(
                            '#',
                        ),
                        Some(
                            'n',
                        ),
                        Some(
                            'b',
                        ),
                        Some(
                            ':',
                        ),
                        Some(
                            't',
                        ),
                        Some(
                            '\\',
                        ),
                        Some(
                            'u',
                        ),
                        Some(
                            'f',
                        ),
                    },
                    found: Some(
                        'v',
                    ),
                    label: None,
                },
            ],
        )
        ",
        );
    }

    #[ignore]
    #[test]
    fn it_parse_invalid_unicode_in_header_key() {
        let test_str = "GET https://example.org\nkey\\uFFFT: thisshoulderror";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        "#,
        );
    }
}

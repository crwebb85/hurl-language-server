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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/protected",
                                ),
                            ],
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
                                            "Basic Ym9iOnNlY3JldA==",
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/protected",
                                ),
                            ],
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
                                            "Basic Ym9iOnNlY3JldA==",
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "this:value:has:colons",
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "thekeyhadescapedcolons",
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "thekeyhasescaped\\backslash",
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

    #[test]
    fn it_errors_header_key_with_empty_template() {
        //TODO make this a recoverable error that warns the user that if they probably forgot to
        //add the template contents (this warning can only be done for header templates since
        //for other interpolated string locations the curly brackets are valid text
        let test_str = "GET https://example.org\nkey-{{ }}: dummyvalue";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r"
        Err(
            [
                Simple {
                    span: 30..31,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        ' ',
                    ),
                    label: None,
                },
            ],
        )
        ",
        );
    }

    #[test]
    fn it_parses_header_key_template() {
        let test_str = "GET https://example.org\nkey-{{env}}: dummyvalue";
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "key-",
                                        ),
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "env",
                                                    ),
                                                    filters: [],
                                                },
                                            },
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            "dummyvalue",
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
    fn it_parses_header_value_with_emoji() {
        let test_str = "GET https://example.org\nkey: valuewithemoji\u{1F600}";
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "valuewithemoji😀",
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
    fn it_parses_empty_header_value_template_as_string() {
        let test_str = "GET https://example.org\nkey: dummyvalue-{{ }}";
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "dummyvalue-",
                                        ),
                                        Str(
                                            "{{",
                                        ),
                                        Str(
                                            " ",
                                        ),
                                        Str(
                                            "}}",
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
    fn it_parses_header_value_template_end_a_non_template_bracket() {
        let test_str = "GET https://example.org\nkey: dummy{v}alue-{{ }}";
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "dummy",
                                        ),
                                        Str(
                                            "{",
                                        ),
                                        Str(
                                            "v",
                                        ),
                                        Str(
                                            "}",
                                        ),
                                        Str(
                                            "alue-",
                                        ),
                                        Str(
                                            "{{",
                                        ),
                                        Str(
                                            " ",
                                        ),
                                        Str(
                                            "}}",
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
    fn it_parses_decode_filter_in_header_value() {
        let test_str = "GET https://example.org/cn\nkey: {{apikey decode \"gb2312\"}}";

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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/cn",
                                ),
                            ],
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
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "apikey",
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
                                            },
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
    fn it_parses_template_header_value_template_with_whitespace_after_colon() {
        let test_str = "GET https://example.org/cn\nkey: {{apikey}}";

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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/cn",
                                ),
                            ],
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
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "apikey",
                                                    ),
                                                    filters: [],
                                                },
                                            },
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
    fn it_errors_from_newline_before_value_of_key_value_pair() {
        let test_str = "GET https://example.org/cn\nkey:\ndummy_key";

        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r"
        Err(
            [
                Simple {
                    span: 31..32,
                    reason: Unexpected,
                    expected: {
                        Some(
                            '\\',
                        ),
                        Some(
                            '{',
                        ),
                    },
                    found: Some(
                        '\n',
                    ),
                    label: None,
                },
            ],
        )
        ",
        );
    }

    #[test]
    fn it_parses_template_header_value_template_without_whitespace_after_colon() {
        let test_str = "GET https://example.org/cn\nkey:{{apikey}}";

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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/cn",
                                ),
                            ],
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
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "apikey",
                                                    ),
                                                    filters: [],
                                                },
                                            },
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
    fn it_parses_url_decode_filter_in_header_value() {
        let test_str = "GET https://example.org/cn\nkey: {{apikey urlDecode}}";

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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/cn",
                                ),
                            ],
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
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "apikey",
                                                    ),
                                                    filters: [
                                                        UrlDecode,
                                                    ],
                                                },
                                            },
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
    fn it_parses_recursive_templates() {
        let test_str =
            "GET https://example.org/cn\nkey: {{apikey urlDecode split \"{{seperator}}\"}}";

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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org/cn",
                                ),
                            ],
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
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "apikey",
                                                    ),
                                                    filters: [
                                                        UrlDecode,
                                                        Split {
                                                            sep: InterpolatedString {
                                                                parts: [
                                                                    Template(
                                                                        Template {
                                                                            expr: Expr {
                                                                                variable: VariableName(
                                                                                    "seperator",
                                                                                ),
                                                                                filters: [],
                                                                            },
                                                                        },
                                                                    ),
                                                                ],
                                                            },
                                                        },
                                                    ],
                                                },
                                            },
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
    fn it_parses_header_value_template() {
        let test_str = "GET https://example.org\nkey: dummyvalue-{{apikey}}";
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
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
                                            "dummyvalue-",
                                        ),
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: VariableName(
                                                        "apikey",
                                                    ),
                                                    filters: [],
                                                },
                                            },
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
    fn it_parses_header_value_template_functions() {
        let test_str = "GET https://example.org\nmessage: {{newUuid}}";
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
                        url: InterpolatedString {
                            parts: [
                                Str(
                                    "https://example.org",
                                ),
                            ],
                        },
                        header: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "message",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Template(
                                            Template {
                                                expr: Expr {
                                                    variable: FunctionName(
                                                        "newUuid",
                                                    ),
                                                    filters: [],
                                                },
                                            },
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

    // text::keyword("getEnv").to(Variable::FunctionName("getEnv".to_owned())),
    // text::keyword("newDate").to(Variable::FunctionName("newDate".to_owned())),
    // text::keyword("newUuid").to(Variable::FunctionName("newUuid".to_owned()))

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

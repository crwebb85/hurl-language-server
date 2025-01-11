#[cfg(test)]
mod tests {
    use crate::parser::parser::ast_parser;
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
            @r#"
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
                    label: Some(
                        "value_escaped_char",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_header_key_with_empty_template() {
        //TODO make this a recoverable error that warns the user that they probably forgot to
        //add the template contents (this warning can only be done for header templates since
        //for other interpolated string locations the curly brackets are valid text
        let test_str = "GET https://example.org\nkey-{{ }}: dummyvalue";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 30..31,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        ' ',
                    ),
                    label: Some(
                        "template",
                    ),
                },
            ],
        )
        "#,
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
            @r#"
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
                    label: Some(
                        "key_value",
                    ),
                },
            ],
        )
        "#,
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
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
                        request_sections: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_multiple_headers() {
        let test_str = "GET https://example.org\nmessage: {{newUuid}}\nkey: {{apikey}}";
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
                        request_sections: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_invalid_variable_name_in_header() {
        //TODO add diagnostic warnings for this
        let test_str = "GET https://example.org\nkey: {{api-key}}";
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
                                            "{{",
                                        ),
                                        Str(
                                            "api-key",
                                        ),
                                        Str(
                                            "}}",
                                        ),
                                    ],
                                },
                            },
                        ],
                        request_sections: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
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

    #[test]
    fn it_parses_option_variables() {
        let test_str = "GET https://{{host}}/{{id}}/status\n[Options]\nvariable: host=example.net\nvariable: id=1234";
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
                                    "https://",
                                ),
                                Template(
                                    Template {
                                        expr: Expr {
                                            variable: VariableName(
                                                "host",
                                            ),
                                            filters: [],
                                        },
                                    },
                                ),
                                Str(
                                    "/",
                                ),
                                Template(
                                    Template {
                                        expr: Expr {
                                            variable: VariableName(
                                                "id",
                                            ),
                                            filters: [],
                                        },
                                    },
                                ),
                                Str(
                                    "/status",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        Variable(
                                            VariableDefinitionOption {
                                                name: "host",
                                                value: String(
                                                    InterpolatedString {
                                                        parts: [
                                                            Str(
                                                                "example.net",
                                                            ),
                                                        ],
                                                    },
                                                ),
                                            },
                                        ),
                                        Variable(
                                            VariableDefinitionOption {
                                                name: "id",
                                                value: Integer(
                                                    1234,
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    // text::keyword("connect-timeout").to(RequestDurationOption::ConnectTimeout),
    // text::keyword("delay").to(RequestDurationOption::Delay),
    // text::keyword("retry-interval").to(RequestDurationOption::RetryInterval),
    #[test]
    fn it_parses_boolean_options() {
        let test_str = "GET https://example.com\n[Options]\ncompressed: true\nlocation: true\nlocation-trusted: true\nhttp1.0: false\nhttp1.1: false\nhttp2: false\nhttp3: true\ninsecure: false\nipv4: false\nipv6: true\nnetrc: true\nnetrc-optional: true\npath-as-is: true\nskip: false\nverbose: true\nvery-verbose: true";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        Compressed(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        Location(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        LocationTrusted(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        Http10(
                                            Literal(
                                                false,
                                            ),
                                        ),
                                        Http11(
                                            Literal(
                                                false,
                                            ),
                                        ),
                                        Http2(
                                            Literal(
                                                false,
                                            ),
                                        ),
                                        Http3(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        Insecure(
                                            Literal(
                                                false,
                                            ),
                                        ),
                                        Ipv4(
                                            Literal(
                                                false,
                                            ),
                                        ),
                                        Ipv6(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        Netrc(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        NetrcOptional(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        PathAsIs(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        Skip(
                                            Literal(
                                                false,
                                            ),
                                        ),
                                        Verbose(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                        VeryVerbose(
                                            Literal(
                                                true,
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_duration_options_with_templates() {
        let test_str =
            "GET https://example.com\n[Options]\nconnect-timeout: {{connectTimeout}}\ndelay: {{delay}}\nretry-interval: {{retryInterval}}";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        ConnectTimeout(
                                            Template(
                                                Template {
                                                    expr: Expr {
                                                        variable: VariableName(
                                                            "connectTimeout",
                                                        ),
                                                        filters: [],
                                                    },
                                                },
                                            ),
                                        ),
                                        Delay(
                                            Template(
                                                Template {
                                                    expr: Expr {
                                                        variable: VariableName(
                                                            "delay",
                                                        ),
                                                        filters: [],
                                                    },
                                                },
                                            ),
                                        ),
                                        RetryInterval(
                                            Template(
                                                Template {
                                                    expr: Expr {
                                                        variable: VariableName(
                                                            "retryInterval",
                                                        ),
                                                        filters: [],
                                                    },
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_duration_options_with_default_unit() {
        let test_str =
            "GET https://example.com\n[Options]\nconnect-timeout: 5\ndelay: 4\nretry-interval: 500";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        ConnectTimeout(
                                            Literal(
                                                Duration {
                                                    duration: 5,
                                                    unit: None,
                                                },
                                            ),
                                        ),
                                        Delay(
                                            Literal(
                                                Duration {
                                                    duration: 4,
                                                    unit: None,
                                                },
                                            ),
                                        ),
                                        RetryInterval(
                                            Literal(
                                                Duration {
                                                    duration: 500,
                                                    unit: None,
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_duration_options_with_default_s_unit() {
        let test_str =
            "GET https://example.com\n[Options]\nconnect-timeout: 5s\ndelay: 4s\nretry-interval: 500s";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        ConnectTimeout(
                                            Literal(
                                                Duration {
                                                    duration: 5,
                                                    unit: Some(
                                                        Second,
                                                    ),
                                                },
                                            ),
                                        ),
                                        Delay(
                                            Literal(
                                                Duration {
                                                    duration: 4,
                                                    unit: Some(
                                                        Second,
                                                    ),
                                                },
                                            ),
                                        ),
                                        RetryInterval(
                                            Literal(
                                                Duration {
                                                    duration: 500,
                                                    unit: Some(
                                                        Second,
                                                    ),
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_duration_options_with_default_ms_unit() {
        let test_str =
            "GET https://example.com\n[Options]\nconnect-timeout: 5ms\ndelay: 4ms\nretry-interval: 500ms";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        ConnectTimeout(
                                            Literal(
                                                Duration {
                                                    duration: 5,
                                                    unit: Some(
                                                        Millisecond,
                                                    ),
                                                },
                                            ),
                                        ),
                                        Delay(
                                            Literal(
                                                Duration {
                                                    duration: 4,
                                                    unit: Some(
                                                        Millisecond,
                                                    ),
                                                },
                                            ),
                                        ),
                                        RetryInterval(
                                            Literal(
                                                Duration {
                                                    duration: 500,
                                                    unit: Some(
                                                        Millisecond,
                                                    ),
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_duration_options_with_default_m_unit() {
        let test_str =
            "GET https://example.com\n[Options]\nconnect-timeout: 5m\ndelay: 4m\nretry-interval: 500m";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        ConnectTimeout(
                                            Literal(
                                                Duration {
                                                    duration: 5,
                                                    unit: Some(
                                                        Minute,
                                                    ),
                                                },
                                            ),
                                        ),
                                        Delay(
                                            Literal(
                                                Duration {
                                                    duration: 4,
                                                    unit: Some(
                                                        Minute,
                                                    ),
                                                },
                                            ),
                                        ),
                                        RetryInterval(
                                            Literal(
                                                Duration {
                                                    duration: 500,
                                                    unit: Some(
                                                        Minute,
                                                    ),
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_retry_interval_option_with_default_unit() {
        let test_str = "GET https://example.com\n[Options]\nretry-interval: 500";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        RetryInterval(
                                            Literal(
                                                Duration {
                                                    duration: 500,
                                                    unit: None,
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_single_duration_options_with_default_unit() {
        let test_str = "GET https://example.com\n[Options]\ndelay: 4";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        Delay(
                                            Literal(
                                                Duration {
                                                    duration: 4,
                                                    unit: None,
                                                },
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_integer_options() {
        let test_str =
            "GET https://example.com\n[Options]\nlimit-rate: 59\nmax-redirs: 109\nrepeat: 10\nretry: 5";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        LimitRate(
                                            Literal(
                                                59,
                                            ),
                                        ),
                                        MaxRedirs(
                                            Literal(
                                                109,
                                            ),
                                        ),
                                        Repeat(
                                            Literal(
                                                10,
                                            ),
                                        ),
                                        Retry(
                                            Literal(
                                                5,
                                            ),
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn it_parses_largest_valid_integer_option_for_usize_64() {
        let test_str = format!(
            "GET https://example.com\n[Options]\nlimit-rate: {}",
            u64::MAX,
        );

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        LimitRate(
                                            Literal(
                                                18446744073709551615,
                                            ),
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    #[test]
    fn it_parses_big_integer_option_usize_64() {
        //18446744073709551616 is just outside the range of numbers for usize 64
        let test_str = "GET https://example.com\n[Options]\nlimit-rate: 18446744073709551616";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        LimitRate(
                                            BigInteger(
                                                "18446744073709551616",
                                            ),
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }

    #[cfg(any(target_pointer_width = "64", target_pointer_width = "32"))]
    #[test]
    fn it_parses_largest_valid_integer_option_for_usize_32() {
        let test_str = format!(
            "GET https://example.com\n[Options]\nlimit-rate: {}",
            u32::MAX,
        );

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        LimitRate(
                                            Literal(
                                                4294967295,
                                            ),
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_value_string_options() {
        let test_str = "GET https://example.com\n[Options]\naws-sigv4: aws:amz:eu-central-1:sts\nconnect-to: example.com:8000:127.0.0.1:8080\nnetrc-file: ~/.netrc\nproxy: example.proxy:8050\nresolve: example.com:8000:127.0.0.1\nunix-socket: sock\nuser: joe=secret";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        AwsSigv4(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "aws:amz:eu-central-1:sts",
                                                    ),
                                                ],
                                            },
                                        ),
                                        ConnectTo(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "example.com:8000:127.0.0.1:8080",
                                                    ),
                                                ],
                                            },
                                        ),
                                        NetrcFile(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "~/.netrc",
                                                    ),
                                                ],
                                            },
                                        ),
                                        Proxy(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "example.proxy:8050",
                                                    ),
                                                ],
                                            },
                                        ),
                                        Resolve(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "example.com:8000:127.0.0.1",
                                                    ),
                                                ],
                                            },
                                        ),
                                        UnixSocket(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "sock",
                                                    ),
                                                ],
                                            },
                                        ),
                                        User(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "joe=secret",
                                                    ),
                                                ],
                                            },
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_value_string_options_with_templates() {
        let test_str = "GET https://{{host}}:{{port}}\n[Options]\naws-sigv4: {{aws}}\nconnect-to: {{host}}:{{port}}:127.0.0.1:8080\nnetrc-file: {{filepath}}\nproxy: {{proxyhost}}:8050\nresolve: {{host}}:{{port}}:127.0.0.1\nunix-socket: {{socket}}\nuser: {{user}}={{password}}";

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
                                    "https://",
                                ),
                                Template(
                                    Template {
                                        expr: Expr {
                                            variable: VariableName(
                                                "host",
                                            ),
                                            filters: [],
                                        },
                                    },
                                ),
                                Str(
                                    ":",
                                ),
                                Template(
                                    Template {
                                        expr: Expr {
                                            variable: VariableName(
                                                "port",
                                            ),
                                            filters: [],
                                        },
                                    },
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        AwsSigv4(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "aws",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        ConnectTo(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "host",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                    Str(
                                                        ":",
                                                    ),
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "port",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                    Str(
                                                        ":127.0.0.1:8080",
                                                    ),
                                                ],
                                            },
                                        ),
                                        NetrcFile(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "filepath",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        Proxy(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "proxyhost",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                    Str(
                                                        ":8050",
                                                    ),
                                                ],
                                            },
                                        ),
                                        Resolve(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "host",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                    Str(
                                                        ":",
                                                    ),
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "port",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                    Str(
                                                        ":127.0.0.1",
                                                    ),
                                                ],
                                            },
                                        ),
                                        UnixSocket(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "socket",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        User(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "user",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                    Str(
                                                        "=",
                                                    ),
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "password",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_filename_options() {
        let test_str = "GET https://example.com\n[Options]\ncacert: /etc/cert.pem\nkey: .ssh/id_rsa.pub\noutput: ./myreport";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        Cacert(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "/etc/cert.pem",
                                                    ),
                                                ],
                                            },
                                        ),
                                        Key(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        ".ssh/id_rsa.pub",
                                                    ),
                                                ],
                                            },
                                        ),
                                        Output(
                                            InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "./myreport",
                                                    ),
                                                ],
                                            },
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_filename_options_with_templates() {
        let test_str = "GET https://example.com\n[Options]\ncacert: {{certfilepath}}\nkey: {{keyfilepath}}\noutput: {{reportfilepath}}";

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
                                    "https://example.com",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            OptionsSection(
                                RequestOptionsSection {
                                    options: [
                                        Cacert(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "certfilepath",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        Key(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "keyfilepath",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                        Output(
                                            InterpolatedString {
                                                parts: [
                                                    Template(
                                                        Template {
                                                            expr: Expr {
                                                                variable: VariableName(
                                                                    "reportfilepath",
                                                                ),
                                                                filters: [],
                                                            },
                                                        },
                                                    ),
                                                ],
                                            },
                                        ),
                                    ],
                                },
                            ),
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
    fn it_parses_query_string_params() {
        let test_str =
            "GET http://localhost:3000/api/search\n[QueryStringParams]\nq: 1982\nsort: name";
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
                                    "http://localhost:3000/api/search",
                                ),
                            ],
                        },
                        header: [],
                        request_sections: [
                            QueryStringParamsSection(
                                QueryStringParamsSection {
                                    queries: [
                                        KeyValue {
                                            key: InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "q",
                                                    ),
                                                ],
                                            },
                                            value: InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "1982",
                                                    ),
                                                ],
                                            },
                                        },
                                        KeyValue {
                                            key: InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "sort",
                                                    ),
                                                ],
                                            },
                                            value: InterpolatedString {
                                                parts: [
                                                    Str(
                                                        "name",
                                                    ),
                                                ],
                                            },
                                        },
                                    ],
                                },
                            ),
                        ],
                    },
                    response: None,
                },
            ],
        )
        "#,
        );
    }
}

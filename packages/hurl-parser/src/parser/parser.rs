use super::body::body_parser;
use super::key_value::key_value_parser;
use super::method::method_line_parser;
use super::primitives::lt_parser;
use super::request_section::request_section_parser;
use super::types::{Ast, Entry, KeyValue, Request, RequestSection};
use chumsky::prelude::*;

fn request_parser<'a>() -> impl Parser<'a, &'a str, Request, extra::Err<Rich<'a, char>>> + Clone {
    let header_line = key_value_parser().then_ignore(lt_parser());
    let request = method_line_parser(false)
        .then(header_line.repeated().collect::<Vec<KeyValue>>())
        .then(
            request_section_parser()
                .repeated()
                .collect::<Vec<RequestSection>>(),
        )
        .then(body_parser().or_not())
        .map(
            |((((method_value, url_value_string), headers), request_sections), body)| Request {
                method: method_value,
                url: url_value_string,
                header: headers,
                request_sections,
                body,
            },
        )
        .labelled("request");
    request.boxed()
}

pub fn ast_parser<'a>() -> impl Parser<'a, &'a str, Ast, extra::Err<Rich<'a, char>>> + Clone {
    let entry = request_parser()
        .map(|request_value| Entry {
            request: Box::new(request_value),
            response: None,
        })
        .labelled("entry")
        .boxed();

    lt_parser()
        .or_not()
        .ignore_then(
            entry
                .clone()
                .recover_with(skip_then_retry_until(
                    //Skip the line and then retry parsing the next line as an entry
                    none_of("\n").repeated().then(just("\n")).ignored(),
                    end(),
                ))
                .repeated()
                .collect::<Vec<Entry>>()
                .recover_with(via_parser(entry.repeated().collect::<Vec<Entry>>())),
        )
        .map(|entries| Ast { entries })
        .boxed()
}

pub fn parse_ast<'a>(document: &'a str) -> (Option<Ast>, Vec<Rich<'a, char>>) {
    let (ast, errs) = ast_parser().parse(document).into_output_errors();
    (ast, errs)
}

#[cfg(test)]
mod ast_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_recovers_from_invalid_method_letter_case_in_entries() {
        let test_str = r#" 
    GeT https://example.org
    GET https://example.org
    post https://example.org
    pOst https://example.org
    POST https://example.org
            "#;
        assert_debug_snapshot!(
        parse_ast(test_str),
            @r#"
        (
            Some(
                Ast {
                    entries: [
                        Entry {
                            request: Request {
                                method: Method {
                                    value: "GeT",
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
                                body: None,
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
                                            "https://example.org",
                                        ),
                                    ],
                                },
                                header: [],
                                request_sections: [],
                                body: None,
                            },
                            response: None,
                        },
                        Entry {
                            request: Request {
                                method: Method {
                                    value: "post",
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
                                body: None,
                            },
                            response: None,
                        },
                        Entry {
                            request: Request {
                                method: Method {
                                    value: "pOst",
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
                                body: None,
                            },
                            response: None,
                        },
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
                                body: None,
                            },
                            response: None,
                        },
                    ],
                },
            ),
            [
                Invalid character 'e'. Method must be ascii uppercase. at 6..9,
                Invalid character 'p'. Method must be ascii uppercase. at 62..66,
                Invalid character 'p'. Method must be ascii uppercase. at 91..95,
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_recovers_from_invalid_syntax_in_entries() {
        //This should parse the 5 valid entries and skip invalid ones until the next valid entry
        let test_str = r#" 
    GET https://example.org
    * https://example.org
    GET https://example.org
    POST https://example.org
    * https://example.org
    *
    #this is a comment
    POST https://example.org
    POST https://example.org
            "#;
        assert_debug_snapshot!(
        parse_ast(test_str),
            @r#"
        (
            Some(
                Ast {
                    entries: [
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
                                body: None,
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
                                            "https://example.org",
                                        ),
                                    ],
                                },
                                header: [],
                                request_sections: [],
                                body: None,
                            },
                            response: None,
                        },
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
                                body: None,
                            },
                            response: None,
                        },
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
                                body: None,
                            },
                            response: None,
                        },
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
                                body: None,
                            },
                            response: None,
                        },
                    ],
                },
            ),
            [
                found end of input at 34..35 expected something else,
                found end of input at 117..118 expected something else,
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_multiple_entries_with_json_request() {
        let test_str = r#" 
    POST https://example.org
    {       
        "id": "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
        "pets": [
            "cat",
            "dog",
            "hampster"
        ]
    }

    POST https://example.org
    {       
        "id": "bde6c63f-eebe-4cae-b955-d128b5d2444d",
        "pets": [
            "cat",
            "dog"
        ]
    }
            "#;
        assert_debug_snapshot!(
        parse_ast(test_str),
            @r#"
        (
            Some(
                Ast {
                    entries: [
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
                                body: Some(
                                    Body {
                                        bytes: JsonValue(
                                            Object(
                                                [
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "id",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
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
                                                                InterpolatedString(
                                                                    InterpolatedString {
                                                                        parts: [
                                                                            Str(
                                                                                "hampster",
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
                            response: None,
                        },
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
                                body: Some(
                                    Body {
                                        bytes: JsonValue(
                                            Object(
                                                [
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "id",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "bde6c63f-eebe-4cae-b955-d128b5d2444d",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
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
                            response: None,
                        },
                    ],
                },
            ),
            [],
        )
        "#,
        );
    }

    #[test]
    fn it_recovers_error_in_first_entries_json_request() {
        let test_str = r#" 
    POST https://example.org
    {       
        "id": "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
        "pets": [
            "cat",
            "dog",
            "hampster"
        
    }

    POST https://example.org
    {       
        "id": "bde6c63f-eebe-4cae-b955-d128b5d2444d",
        "pets": [
            "cat",
            "dog"
        ]
    }
            "#;
        assert_debug_snapshot!(
        parse_ast(test_str),
            @r#"
        (
            Some(
                Ast {
                    entries: [
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
                                body: Some(
                                    Body {
                                        bytes: JsonValue(
                                            Object(
                                                [
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "id",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "pets",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "cat",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
                                                ],
                                            ),
                                        ),
                                    },
                                ),
                            },
                            response: None,
                        },
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
                                body: Some(
                                    Body {
                                        bytes: JsonValue(
                                            Object(
                                                [
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "id",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "bde6c63f-eebe-4cae-b955-d128b5d2444d",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
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
                            response: None,
                        },
                    ],
                },
            ),
            [
                found ''}'' at 190..191 expected '','', or '']'',
                found '','' at 152..153 expected '':'',
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_recovers_error_in_json_request_with_commented_closing_bracket() {
        let test_str = r#" 
    POST https://example.org
    {       
        "id": "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
        "pet": "cat"
    #}

    POST https://example.org
    {       
        "id": "55b42346-02be-4cf3-824c-b2dcdf5f7512",
        "pet": "dog"
    }

    "#;
        assert_debug_snapshot!(
        parse_ast(test_str),
            @r#"
        (
            Some(
                Ast {
                    entries: [
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
                                body: Some(
                                    Body {
                                        bytes: JsonValue(
                                            Object(
                                                [
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "id",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "d89e270c-5f26-4906-b305-c9e3cc2a0a24",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "pet",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "cat",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
                                                ],
                                            ),
                                        ),
                                    },
                                ),
                            },
                            response: None,
                        },
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
                                body: Some(
                                    Body {
                                        bytes: JsonValue(
                                            Object(
                                                [
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "id",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "55b42346-02be-4cf3-824c-b2dcdf5f7512",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
                                                    JsonKeyValue {
                                                        key: InterpolatedString {
                                                            parts: [
                                                                Str(
                                                                    "pet",
                                                                ),
                                                            ],
                                                        },
                                                        value: InterpolatedString(
                                                            InterpolatedString {
                                                                parts: [
                                                                    Str(
                                                                        "dog",
                                                                    ),
                                                                ],
                                                            },
                                                        ),
                                                    },
                                                ],
                                            ),
                                        ),
                                    },
                                ),
                            },
                            response: None,
                        },
                    ],
                },
            ),
            [
                found ''#'' at 123..124 expected '','', or ''}'',
            ],
        )
        "#,
        );
    }
}

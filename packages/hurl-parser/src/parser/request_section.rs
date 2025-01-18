use super::filename::filename_parser;
use super::key_value::{key_parser, key_value_parser};
use super::options::option_parser;
use super::primitives::{lt_parser, sp_parser};
use super::types::{
    BasicAuthSection, CookiesSection, FileKeyValue, FileValue, FormParamsSection,
    MultipartFormDataSection, MultipartFormParam, QueryStringParamsSection, RequestOptionsSection,
    RequestSection,
};
use chumsky::prelude::*;

pub fn file_param_parser() -> impl Parser<char, FileKeyValue, Error = Simple<char>> + Clone {
    let sp = sp_parser();
    let key = key_parser();
    let filename = filename_parser();

    let file_content_type = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric() || c == &'/' || c == &'+' || c == &'-'
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .labelled("file_content_type");

    let file_value = just("file,")
        .then(filename.clone())
        .then_ignore(just(';'))
        .padded_by(sp.clone().repeated())
        .then(file_content_type.or_not())
        .map(|((_, filename), content_type)| FileValue {
            filename,
            content_type,
        });

    let file_param = key
        .clone()
        .padded_by(sp.clone().repeated())
        .then_ignore(just(':'))
        .padded_by(sp.clone().repeated())
        .then(file_value)
        .map(|(key, value)| FileKeyValue { key, value })
        .labelled("file_key_value");
    file_param
}

pub fn request_section_parser() -> impl Parser<char, RequestSection, Error = Simple<char>> + Clone {
    let sp = sp_parser();
    let lt = lt_parser();
    let key_value = key_value_parser();
    let option = option_parser();
    let key_values = key_value.clone().then_ignore(lt.clone()).repeated();

    let basic_auth_section = sp
        .clone()
        .repeated()
        .ignored()
        .then_ignore(just("[BasicAuth]"))
        .then_ignore(lt.clone().repeated())
        .then_ignore(sp.clone().repeated())
        .then(key_values.clone().validate(|key_values, span, emit| {
            if key_values.len() > 1 {
                emit(Simple::custom(
                    span,
                    "Basic Auth can only be defined for one user.",
                ));
            }
            key_values
        }))
        .map(|(_, auth_key_values)| {
            RequestSection::BasicAuthSection(BasicAuthSection {
                key_values: auth_key_values,
            })
        });

    let query_string_params_section = sp
        .clone()
        .repeated()
        .ignored()
        .then_ignore(just("[QueryStringParams]").or(just("[Query]")))
        .then_ignore(lt.clone().repeated())
        .then_ignore(sp.clone().repeated())
        .then(key_values.clone())
        .map(|(_, query_key_values)| {
            RequestSection::QueryStringParamsSection(QueryStringParamsSection {
                queries: query_key_values,
            })
        });

    let form_params_section = sp
        .clone()
        .repeated()
        .ignored()
        .then_ignore(just("[FormParams]").or(just("[Form]")))
        .then_ignore(lt.clone().repeated())
        .then_ignore(sp.clone().repeated())
        .then(key_values.clone())
        .map(|(_, form_params)| {
            RequestSection::FormParamsSection(FormParamsSection {
                params: form_params,
            })
        });

    let file_param = file_param_parser().map(MultipartFormParam::FileParam);
    let multipart_form_param = file_param
        .or(key_value.map(MultipartFormParam::KeyValueParam))
        .then_ignore(lt.clone());

    let multipart_form_data_section = sp
        .clone()
        .repeated()
        .ignored()
        .then_ignore(just("[MultipartFormData]").or(just("[Multipart]")))
        .then_ignore(lt.clone().repeated())
        .then_ignore(sp.clone().repeated())
        .then(multipart_form_param.repeated())
        .map(|(_, file_params)| {
            RequestSection::MultipartFormDataSection(MultipartFormDataSection {
                params: file_params,
            })
        });

    let cookies_section = sp
        .clone()
        .repeated()
        .ignored()
        .then_ignore(just("[Cookies]"))
        .then_ignore(lt.clone().repeated())
        .then_ignore(sp.clone().repeated())
        .then(key_values.clone())
        .map(|(_, cookies_key_value)| {
            RequestSection::CookiesSection(CookiesSection {
                cookies: cookies_key_value,
            })
        });

    let options_section = sp
        .clone()
        .repeated()
        .ignored()
        .then_ignore(just("[Options]"))
        .then_ignore(lt.clone().repeated())
        .then_ignore(sp.clone().repeated())
        .then(option.repeated())
        .map(|(_, options)| RequestSection::OptionsSection(RequestOptionsSection { options }));

    let request_section = basic_auth_section
        .or(query_string_params_section)
        .or(form_params_section)
        .or(multipart_form_data_section)
        .or(cookies_section)
        .or(options_section);
    // TODO and an unknown section for error handling
    // .or(unknown_section);

    request_section
}

//TODO sections can only be defined once per entry's request section. So you can't have [BasicAuth] defined
//twice and so should be a diagnostic error.

#[cfg(test)]
mod request_section_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_basic_auth_section() {
        let test_str = "[BasicAuth]\njoe: secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            BasicAuthSection(
                BasicAuthSection {
                    key_values: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "joe",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "secret",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_errors_basic_auth_section_with_multiple_key_values() {
        //TODO improve error recovery since it would be nice still have a working AST
        let test_str = "[BasicAuth]\njoe: secret\nalice:secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 12..36,
                    reason: Custom(
                        "Basic Auth can only be defined for one user.",
                    ),
                    expected: {},
                    found: None,
                    label: None,
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_parses_basic_auth_with_extra_line_terminators_and_spacing() {
        let test_str = "[BasicAuth]\n\n #a nice comment\n  joe: secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            BasicAuthSection(
                BasicAuthSection {
                    key_values: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "joe",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "secret",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_basic_auth_with_extra_spacing_before_section() {
        let test_str = "  [BasicAuth] joe: secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            BasicAuthSection(
                BasicAuthSection {
                    key_values: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "joe",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "secret",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section() {
        let test_str = "[QueryStringParams]\nsearch: {{my-search}}\norder: desc\ncount: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            QueryStringParamsSection(
                QueryStringParamsSection {
                    queries: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "search",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "my-search",
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
                                        "order",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "desc",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "count",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "420",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section_with_section_alias() {
        let test_str = "[Query]\nsearch: {{my-search}}\norder: desc\ncount: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            QueryStringParamsSection(
                QueryStringParamsSection {
                    queries: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "search",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "my-search",
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
                                        "order",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "desc",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "count",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "420",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[ignore]
    #[test]
    fn it_parses_query_string_params_section_with_extra_spaces_and_line_terminators() {
        //TODO fix multiple line terminator parsing
        //TODO fix multiple leading space parsing before key values
        let test_str = "  [QueryStringParams]\n  search: {{my-search}}\n #we need descent\n   order: desc   \n\n\n#420 xD!\n count: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            QueryStringParamsSection(
                QueryStringParamsSection {
                    queries: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "search",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "my-search",
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
                                        "order",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "desc",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "count",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "420",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[ignore]
    #[test]
    fn it_parses_query_string_params_section_with_extra_spaces() {
        //TODO fix multiple leading space parsing before key values
        let test_str =
            "  [QueryStringParams]\n  search: {{my-search}}\n    order: desc   \n count: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            QueryStringParamsSection(
                QueryStringParamsSection {
                    queries: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "search",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "my-search",
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
                                        "order",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "desc",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "count",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "420",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[ignore]
    #[test]
    fn it_parses_query_string_params_section_with_line_terminators() {
        //TODO fix multiple line terminator parsing
        let test_str = "[QueryStringParams]\nsearch: {{my-search}}\n #we need descent\norder: desc\n\n\n#420 xD!\ncount: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            QueryStringParamsSection(
                QueryStringParamsSection {
                    queries: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "search",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "my-search",
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
                                        "order",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "desc",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "count",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "420",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_form_params() {
        let test_str = "[FormParams]\ntoken: {{token}}\nemail: john.smith@example.com\naccountNumber: {{accountNumber}}\nenabledEmailNotifications: true";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            FormParamsSection(
                FormParamsSection {
                    params: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "token",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "token",
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
                                        "email",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "john.smith@example.com",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "accountNumber",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "accountNumber",
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
                                        "enabledEmailNotifications",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "true",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_form_params_with_section_alias() {
        let test_str = "[Form]\ntoken: {{token}}\nemail: john.smith@example.com\naccountNumber: {{accountNumber}}\nenabledEmailNotifications: true";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            FormParamsSection(
                FormParamsSection {
                    params: [
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "token",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "token",
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
                                        "email",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "john.smith@example.com",
                                    ),
                                ],
                            },
                        },
                        KeyValue {
                            key: InterpolatedString {
                                parts: [
                                    Str(
                                        "accountNumber",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Template(
                                        Template {
                                            expr: Expr {
                                                variable: VariableName(
                                                    "accountNumber",
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
                                        "enabledEmailNotifications",
                                    ),
                                ],
                            },
                            value: InterpolatedString {
                                parts: [
                                    Str(
                                        "true",
                                    ),
                                ],
                            },
                        },
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_multipart_form_data() {
        let test_str = "[MultipartFormData]\nfield1: value1\nfield2: file,example.txt;\nfield3: file,example.zip; application/zip";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            MultipartFormDataSection(
                MultipartFormDataSection {
                    params: [
                        KeyValueParam(
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "field1",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            "value1",
                                        ),
                                    ],
                                },
                            },
                        ),
                        FileParam(
                            FileKeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "field2",
                                        ),
                                    ],
                                },
                                value: FileValue {
                                    filename: InterpolatedString {
                                        parts: [
                                            Str(
                                                "example.txt",
                                            ),
                                        ],
                                    },
                                    content_type: None,
                                },
                            },
                        ),
                        FileParam(
                            FileKeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "field3",
                                        ),
                                    ],
                                },
                                value: FileValue {
                                    filename: InterpolatedString {
                                        parts: [
                                            Str(
                                                "example.zip",
                                            ),
                                        ],
                                    },
                                    content_type: Some(
                                        "application/zip",
                                    ),
                                },
                            },
                        ),
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_multipart_form_data_with_section_alias() {
        let test_str = "[Multipart]\nfield1: value1\nfield2: file,example.txt;\nfield3: file,example.zip; application/zip";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            MultipartFormDataSection(
                MultipartFormDataSection {
                    params: [
                        KeyValueParam(
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "field1",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            "value1",
                                        ),
                                    ],
                                },
                            },
                        ),
                        FileParam(
                            FileKeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "field2",
                                        ),
                                    ],
                                },
                                value: FileValue {
                                    filename: InterpolatedString {
                                        parts: [
                                            Str(
                                                "example.txt",
                                            ),
                                        ],
                                    },
                                    content_type: None,
                                },
                            },
                        ),
                        FileParam(
                            FileKeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "field3",
                                        ),
                                    ],
                                },
                                value: FileValue {
                                    filename: InterpolatedString {
                                        parts: [
                                            Str(
                                                "example.zip",
                                            ),
                                        ],
                                    },
                                    content_type: Some(
                                        "application/zip",
                                    ),
                                },
                            },
                        ),
                    ],
                },
            ),
        )
        "#,
        );
    }

    #[test]
    fn it_parses_file_param() {
        let test_str = "field2: file,example.txt;";
        assert_debug_snapshot!(
        file_param_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            FileKeyValue {
                key: InterpolatedString {
                    parts: [
                        Str(
                            "field2",
                        ),
                    ],
                },
                value: FileValue {
                    filename: InterpolatedString {
                        parts: [
                            Str(
                                "example.txt",
                            ),
                        ],
                    },
                    content_type: None,
                },
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_file_param_with_content_type() {
        let test_str = "field3: file,example.zip; application/zip";
        assert_debug_snapshot!(
        file_param_parser().then_ignore(end()).parse(test_str),
            @r#"
        Ok(
            FileKeyValue {
                key: InterpolatedString {
                    parts: [
                        Str(
                            "field3",
                        ),
                    ],
                },
                value: FileValue {
                    filename: InterpolatedString {
                        parts: [
                            Str(
                                "example.zip",
                            ),
                        ],
                    },
                    content_type: Some(
                        "application/zip",
                    ),
                },
            },
        )
        "#,
        );
    }
}

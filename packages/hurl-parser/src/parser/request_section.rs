use super::key_value::{key_parser, key_value_parser};
use super::oneline_file::oneline_file_parser;
use super::options::options_parser;
use super::primitives::{lt_parser, sp_parser};
use super::types::{
    BasicAuthSection, CookiesSection, FileKeyValue, FileValue, FormParamsSection, KeyValue,
    MultipartFormDataSection, MultipartFormParam, QueryStringParamsSection, RequestOptionsSection,
    RequestSection,
};
use chumsky::prelude::*;

fn file_param_parser<'a>(
) -> impl Parser<'a, &'a str, FileKeyValue, extra::Err<Rich<'a, char>>> + Clone {
    let file_content_type = any()
        .filter(|c: &char| c.is_ascii_alphanumeric() || c == &'/' || c == &'+' || c == &'-')
        .repeated()
        .at_least(1)
        .collect::<String>()
        .labelled("file_content_type");

    //TODO determine where I need add padding to this
    let file_value = oneline_file_parser()
        .padded_by(sp_parser().repeated())
        .then(file_content_type.padded_by(sp_parser().repeated()).or_not())
        .map(|(filename, content_type)| FileValue {
            filename,
            content_type,
        });

    let file_param = key_parser()
        .padded_by(sp_parser().repeated())
        .then_ignore(just(':').padded_by(sp_parser().repeated()))
        .then(file_value)
        .map(|(key, value)| FileKeyValue { key, value })
        .labelled("file_key_value");

    file_param.boxed()
}

pub fn request_section_parser<'a>(
) -> impl Parser<'a, &'a str, RequestSection, extra::Err<Rich<'a, char>>> + Clone {
    let key_values = key_value_parser()
        .then_ignore(lt_parser())
        .repeated()
        .collect::<Vec<KeyValue>>();

    let basic_auth_section = just("[BasicAuth]")
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(key_values.clone().validate(|key_values, e, emitter| {
            if key_values.len() > 1 {
                emitter.emit(Rich::custom(
                    e.span(),
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

    let query_string_params_section = choice((just("[QueryStringParams]"), just("[Query]")))
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(key_values.clone())
        .map(|(_, query_key_values)| {
            RequestSection::QueryStringParamsSection(QueryStringParamsSection {
                queries: query_key_values,
            })
        });

    let form_params_section = choice((just("[FormParams]"), just("[Form]")))
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(key_values.clone())
        .map(|(_, form_params)| {
            RequestSection::FormParamsSection(FormParamsSection {
                params: form_params,
            })
        });

    let file_param = file_param_parser().map(MultipartFormParam::FileParam);
    let multipart_form_param = file_param
        .or(key_value_parser().map(MultipartFormParam::KeyValueParam))
        .then_ignore(lt_parser());

    let multipart_form_data_section = choice((just("[MultipartFormData]"), just("[Multipart]")))
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(
            multipart_form_param
                .repeated()
                .collect::<Vec<MultipartFormParam>>(),
        )
        .map(|(_, file_params)| {
            RequestSection::MultipartFormDataSection(MultipartFormDataSection {
                params: file_params,
            })
        });

    let cookies_section = just("[Cookies]")
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(key_values.clone())
        .map(|(_, cookies_key_value)| {
            RequestSection::CookiesSection(CookiesSection {
                cookies: cookies_key_value,
            })
        });

    let options_section = just("[Options]")
        .padded_by(sp_parser().repeated())
        .then_ignore(lt_parser())
        .then(options_parser())
        .map(|(_, options)| RequestSection::OptionsSection(RequestOptionsSection { options }));

    let request_section = basic_auth_section
        .or(query_string_params_section)
        .or(form_params_section)
        .or(multipart_form_data_section)
        .or(cookies_section)
        .or(options_section);
    // TODO and an unknown section for error handling
    // .or(unknown_section);

    request_section.boxed()
}

pub fn request_sections_parser<'a>(
) -> impl Parser<'a, &'a str, Vec<RequestSection>, extra::Err<Rich<'a, char>>> + Clone {
    //TODO add tests
    request_section_parser()
        .repeated()
        .collect::<Vec<RequestSection>>()

    //TODO sections can only be defined once per entry's request section. So you can't have [BasicAuth] defined
    //twice and so should be a diagnostic error.
}

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
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
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
        ParseResult {
            output: Some(
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
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "alice",
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
            ),
            errs: [
                Basic Auth can only be defined for one user. at 12..36,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_basic_auth_with_extra_line_terminators_and_spacing() {
        let test_str = "[BasicAuth]\n\n #a nice comment\n  joe: secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_basic_auth_with_extra_spacing_before_section() {
        let test_str = "  [BasicAuth] \n joe: secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section() {
        let test_str = "[QueryStringParams]\nsearch: {{my-search}}\norder: desc\ncount: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section_with_section_alias() {
        let test_str = "[Query]\nsearch: {{my-search}}\norder: desc\ncount: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section_with_extra_spaces_and_line_terminators() {
        let test_str = "  [QueryStringParams]\n  search: {{my-search}}\n #we need descent\n   order: desc   \n\n\n#420 xD!\n count: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
                                            "desc   ",
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section_with_extra_spaces() {
        let test_str =
            "  [QueryStringParams]\n  search: {{my-search}}\n    order: desc   \n count: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
                                            "desc   ",
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_query_string_params_section_with_line_terminators() {
        let test_str = "[QueryStringParams]\nsearch: {{my-search}}\n #we need descent\norder: desc\n\n\n#420 xD!\ncount: 420";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_form_params() {
        let test_str = "[FormParams]\ntoken: {{token}}\nemail: john.smith@example.com\naccountNumber: {{accountNumber}}\nenabledEmailNotifications: true";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_form_params_with_section_alias() {
        let test_str = "[Form]\ntoken: {{token}}\nemail: john.smith@example.com\naccountNumber: {{accountNumber}}\nenabledEmailNotifications: true";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_multipart_form_data() {
        let test_str = "[MultipartFormData]\nfield1: value1\nfield2: file,example.txt;\nfield3: file,example.zip; application/zip";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_multipart_form_data_with_section_alias() {
        let test_str = "[Multipart]\nfield1: value1\nfield2: file,example.txt;\nfield3: file,example.zip; application/zip";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_file_param() {
        let test_str = "field2: file,example.txt;";
        assert_debug_snapshot!(
        file_param_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_file_param_with_content_type() {
        let test_str = "field3: file,example.zip; application/zip";
        assert_debug_snapshot!(
        file_param_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_file_param_with_content_type_and_extra_spaces() {
        let test_str = "field3  :  file,   example.zip  ;   application/zip";
        assert_debug_snapshot!(
        file_param_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_cookie_section() {
        let test_str = "[Cookies]\ntheme: dark\nsessionToken: {{token}}";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                CookiesSection(
                    CookiesSection {
                        cookies: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "theme",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            "dark",
                                        ),
                                    ],
                                },
                            },
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "sessionToken",
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
                        ],
                    },
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_cookie_section_with_extra_withspace_and_line_terminators() {
        let test_str =
            " [Cookies]\n #dark mode is life \n theme: dark\n \n \nsessionToken: {{token}}";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                CookiesSection(
                    CookiesSection {
                        cookies: [
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "theme",
                                        ),
                                    ],
                                },
                                value: InterpolatedString {
                                    parts: [
                                        Str(
                                            "dark",
                                        ),
                                    ],
                                },
                            },
                            KeyValue {
                                key: InterpolatedString {
                                    parts: [
                                        Str(
                                            "sessionToken",
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
                        ],
                    },
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_option_variables() {
        let test_str = "[Options]\nvariable: host=example.net\nvariable: id=1234";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_boolean_options() {
        let test_str = "[Options]\ncompressed: true\nlocation: true\nlocation-trusted: true\nhttp1.0: false\nhttp1.1: false\nhttp2: false\nhttp3: true\ninsecure: false\nipv4: false\nipv6: true\nnetrc: true\nnetrc-optional: true\npath-as-is: true\nskip: false\nverbose: true\nvery-verbose: true";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_templates() {
        let test_str =
            "[Options]\nconnect-timeout: {{connectTimeout}}\ndelay: {{delay}}\nretry-interval: {{retryInterval}}";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_unit() {
        let test_str = "[Options]\nconnect-timeout: 5\ndelay: 4\nretry-interval: 500";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_s_unit() {
        let test_str = "[Options]\nconnect-timeout: 5s\ndelay: 4s\nretry-interval: 500s";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_ms_unit() {
        let test_str = "[Options]\nconnect-timeout: 5ms\ndelay: 4ms\nretry-interval: 500ms";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_m_unit() {
        let test_str = "[Options]\nconnect-timeout: 5m\ndelay: 4m\nretry-interval: 500m";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_retry_interval_option_with_default_unit() {
        let test_str = "[Options]\nretry-interval: 500";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_single_duration_options_with_default_unit() {
        let test_str = "[Options]\ndelay: 4";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_integer_options() {
        let test_str = "[Options]\nlimit-rate: 59\nmax-redirs: 109\nrepeat: 10\nretry: 5";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_largest_valid_integer_option_for_u64() {
        let test_str = format!("[Options]\nlimit-rate: {}", u64::MAX,);
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(&test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_big_integer_option_u64() {
        //18446744073709551616 is just outside the range of numbers for u64
        let test_str = "[Options]\nlimit-rate: 18446744073709551616";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [
                The integer value is larger than 18446744073709551615 and is not valid for 64bit version of hurl at 22..42,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_largest_valid_integer_option_for_u32() {
        let test_str = format!("[Options]\nlimit-rate: {}", u32::MAX,);
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(&test_str),
            @r"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_value_string_options() {
        let test_str = "[Options]\naws-sigv4: aws:amz:eu-central-1:sts\nconnect-to: example.com:8000:127.0.0.1:8080\nnetrc-file: ~/.netrc\nproxy: example.proxy:8050\nresolve: example.com:8000:127.0.0.1\nunix-socket: sock\nuser: joe=secret";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_value_string_options_with_templates() {
        let test_str = "[Options]\naws-sigv4: {{aws}}\nconnect-to: {{host}}:{{port}}:127.0.0.1:8080\nnetrc-file: {{filepath}}\nproxy: {{proxyhost}}:8050\nresolve: {{host}}:{{port}}:127.0.0.1\nunix-socket: {{socket}}\nuser: {{user}}={{password}}";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_filename_options() {
        let test_str = "[Options]\ncacert: /etc/cert.pem\nkey: .ssh/id_rsa.pub\noutput: ./myreport";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_filename_options_with_templates() {
        let test_str =
            "[Options]\ncacert: {{certfilepath}}\nkey: {{keyfilepath}}\noutput: {{reportfilepath}}";
        assert_debug_snapshot!(
        request_section_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }
}

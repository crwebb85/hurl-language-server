use super::body::body_parser;
use super::key_value::{key_value_parser, value_parser};
use super::primitives::{lt_parser, sp_parser};
use super::request_section::request_section_parser;
use super::types::{Ast, Entry, KeyValue, Method, Request, RequestSection};
use chumsky::prelude::*;

fn method_parser<'a>() -> impl Parser<'a, &'a str, Method, extra::Err<Rich<'a, char>>> + Clone {
    let method = text::ident()
        .to_slice()
        .validate(|ident: &str, e, emitter| {
            match ident.find(|c| !char::is_ascii_uppercase(&c)) {
                Some(index) => emitter.emit(Rich::custom(
                    e.span(),
                    format!(
                        "Invalid character '{}'. Method must be ascii uppercase.",
                        ident.chars().nth(index).unwrap() // We know the character is in the index
                    ),
                )),
                None => (),
            };

            Method {
                value: ident.to_string(),
            }
        })
        .padded();
    method.boxed()
}

fn request_parser<'a>() -> impl Parser<'a, &'a str, Request, extra::Err<Rich<'a, char>>> + Clone {
    let header_line = key_value_parser().then_ignore(lt_parser());
    let request = method_parser()
        .padded_by(sp_parser().repeated())
        .then(value_parser())
        .then_ignore(lt_parser())
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
}

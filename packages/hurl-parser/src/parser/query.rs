use super::{
    primitives::sp_parser,
    quoted_string::quoted_string_parser,
    regex::regex_parser,
    types::{CertificateFieldSelector, Query},
};
use chumsky::prelude::*;

pub fn query_parser<'a>() -> impl Parser<'a, &'a str, Query, extra::Err<Rich<'a, char>>> + Clone {
    let query = choice((
        just("status").to(Query::Status),
        just("url").to(Query::Url),
        just("header")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(quoted_string_parser())
            .map(|(_, header_name)| Query::Header(header_name)),
        just("certificate")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(choice((
                //Off spec the grammer doesn't have quotes here but
                //it is required
                just("\"Subject\"").to(CertificateFieldSelector::Subject),
                just("\"Issuer\"").to(CertificateFieldSelector::Issuer),
                just("\"Start-Date\"").to(CertificateFieldSelector::StartDate),
                just("\"Expire-Date\"").to(CertificateFieldSelector::ExpireDate),
                just("\"Serial-Number\"").to(CertificateFieldSelector::Subject),
            )))
            .map(|(_, certificate_field_selector)| Query::Certificate(certificate_field_selector)),
        just("cookie")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(quoted_string_parser())
            .map(|(_, cookie_name)| Query::Cookie(cookie_name)),
        just("body").to(Query::Body),
        just("xpath")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(quoted_string_parser())
            .map(|(_, xpath_selector)| Query::Xpath(xpath_selector)),
        just("jsonpath")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(quoted_string_parser())
            .map(|(_, jsonpath_selector)| Query::JsonPath(jsonpath_selector)),
        just("regex")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(regex_parser(quoted_string_parser()))
            .map(|(_, regex)| Query::Regex(regex)),
        just("variable")
            .then_ignore(sp_parser().repeated().at_least(1))
            .then(quoted_string_parser())
            .map(|(_, variable)| Query::Variable(variable)),
        just("duration").to(Query::Duration),
        just("sha256").to(Query::Sha256),
        just("md5").to(Query::Md5),
        just("bytes").to(Query::Bytes),
    ));

    query.boxed()
}

#[cfg(test)]
mod query_tests {
    use super::*;
    use chumsky::Parser;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_query_status() {
        let test_str = "status";
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Status,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_url() {
        let test_str = "url";
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Url,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_header() {
        let test_str = r#"header "Location""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Header(
                    InterpolatedString {
                        parts: [
                            Str(
                                "Location",
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
    fn it_parses_query_certificate_subject() {
        let test_str = r#"certificate "Subject""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Certificate(
                    Subject,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_certificate_issuer() {
        let test_str = r#"certificate "Issuer""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Certificate(
                    Issuer,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_certificate_expire_date() {
        let test_str = r#"certificate "Expire-Date""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Certificate(
                    ExpireDate,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_certificate_serial_number() {
        let test_str = r#"certificate "Serial-Number""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Certificate(
                    Subject,
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_cookie() {
        let test_str = r#"cookie "LSID""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID",
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
    fn it_parses_query_cookie_value() {
        let test_str = r#"cookie "LSID[Value]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[Value]",
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
    fn it_parses_query_cookie_expires() {
        let test_str = r#"cookie "LSID[Expires]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[Expires]",
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
    fn it_parses_query_cookie_max_age() {
        let test_str = r#"cookie "LSID[Max-Age]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[Max-Age]",
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
    fn it_parses_query_cookie_domain() {
        let test_str = r#"cookie "LSID[Domain]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[Domain]",
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
    fn it_parses_query_cookie_path() {
        let test_str = r#"cookie "LSID[Path]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[Path]",
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
    fn it_parses_query_cookie_secure() {
        let test_str = r#"cookie "LSID[Secure]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[Secure]",
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
    fn it_parses_query_cookie_http_only() {
        let test_str = r#"cookie "LSID[HttpOnly]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[HttpOnly]",
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
    fn it_parses_query_cookie_same_site() {
        let test_str = r#"cookie "LSID[SameSite]""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Cookie(
                    InterpolatedString {
                        parts: [
                            Str(
                                "LSID[SameSite]",
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
    fn it_parses_query_body() {
        let test_str = r#"body"#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Body,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_xpath() {
        let test_str = r#"xpath "normalize-space(//div[@id='pet2'])""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Xpath(
                    InterpolatedString {
                        parts: [
                            Str(
                                "normalize-space(//div[@id='pet2'])",
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
    fn it_parses_query_jsonpath() {
        let test_str = r#"jsonpath "$['type']""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                JsonPath(
                    InterpolatedString {
                        parts: [
                            Str(
                                "$['type']",
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
    fn it_parses_query_regex() {
        let test_str = r#"regex "[0-9]+""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Regex(
                    Interpolated(
                        InterpolatedString {
                            parts: [
                                Str(
                                    "[0-9]+",
                                ),
                            ],
                        },
                    ),
                ),
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_query_variable() {
        let test_str = r#"variable "api_key""#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Variable(
                    InterpolatedString {
                        parts: [
                            Str(
                                "api_key",
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
    fn it_parses_query_duration() {
        let test_str = r#"duration"#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Duration,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_sha256() {
        let test_str = r#"sha256"#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Sha256,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_md5() {
        let test_str = r#"md5"#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Md5,
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_query_bytes() {
        let test_str = r#"bytes"#;
        assert_debug_snapshot!(
        query_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Bytes,
            ),
            errs: [],
        }
        ",
        );
    }
}

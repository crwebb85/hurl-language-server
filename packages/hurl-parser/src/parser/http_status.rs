use super::primitives::{lt_parser, sp_parser};
use super::types::{HttpStatus, HttpVersion};
use chumsky::prelude::*;

fn version_number_parser<'a>(
) -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    let fraction_part = just(".").then(text::digits(10));
    let version_number = text::digits(10)
        .then(fraction_part.or_not())
        .boxed()
        .to_slice()
        .map(|s: &str| s.to_string());

    version_number
}

fn version_parser<'a>() -> impl Parser<'a, &'a str, HttpVersion, extra::Err<Rich<'a, char>>> + Clone
{
    let unknown_version = just("HTTP/")
        .then(version_number_parser())
        .to_slice()
        .validate(|s: &str, e, emitter| {
            emitter.emit(Rich::custom(e.span(), "Unknown http version"));
            HttpVersion::HttpUknown(s.to_string())
        })
        .boxed();

    let version = choice((
        just("HTTP/1.0").to(HttpVersion::Http1_0),
        just("HTTP/1.1").to(HttpVersion::Http1_1),
        just("HTTP/2").to(HttpVersion::Http2),
        just("HTTP/3").to(HttpVersion::Http3),
        unknown_version,
        just("HTTP").to(HttpVersion::Http),
    ))
    .boxed();

    version
}

pub fn http_status_line_parser<'a>(
) -> impl Parser<'a, &'a str, (HttpVersion, HttpStatus), extra::Err<Rich<'a, char>>> + Clone {
    let status = text::digits(10)
        .to_slice()
        .validate(|number: &str, e, emitter| match number.parse::<u64>() {
            Ok(n) => HttpStatus::Code(n),
            Err(_) => {
                emitter.emit(Rich::custom(e.span(), "invalid status: too large"));
                HttpStatus::Invalid
            }
        })
        .or(just('*').to(HttpStatus::Any))
        .or(any()
            .and_is(one_of("#\n").not())
            .repeated()
            .to_slice()
            .validate(|text: &str, e, emitter| {
                if text.is_empty() {
                    emitter.emit(Rich::custom(e.span(), "missing status"));
                    HttpStatus::Missing
                } else {
                    emitter.emit(Rich::custom(e.span(), "invalid status: not a number"));
                    HttpStatus::Invalid
                }
            }));

    let http_status_line = version_parser()
        .then(
            sp_parser()
                .repeated()
                .at_least(1)
                .ignore_then(status)
                .or_not()
                .validate(|status, e, emitter| match status {
                    Some(s) => s,
                    None => {
                        emitter.emit(Rich::custom(e.span(), "missing status"));
                        HttpStatus::Missing
                    }
                }),
        )
        .then_ignore(lt_parser())
        .boxed();

    http_status_line
}

#[cfg(test)]
mod http_status_line_tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_http_200() {
        let test_str = "HTTP 200";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http,
                    Code(
                        200,
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_http1_0_200() {
        let test_str = "HTTP/1.0 200";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http1_0,
                    Code(
                        200,
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_http1_1_200() {
        let test_str = "HTTP/1.1 200";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http1_1,
                    Code(
                        200,
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_http2_200() {
        let test_str = "HTTP/2 200";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http2,
                    Code(
                        200,
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_http3_200() {
        let test_str = "HTTP/3 200";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http3,
                    Code(
                        200,
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_recovers_missing_status() {
        let test_str = "HTTP/3";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http3,
                    Missing,
                ),
            ),
            errs: [
                missing status at 6..6,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_recovers_missing_status_with_whitespace() {
        let test_str = "HTTP/3 ";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http3,
                    Missing,
                ),
            ),
            errs: [
                missing status at 7..7,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_recovers_invalid_version() {
        let test_str = "HTTP/4.5 200";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    HttpUknown(
                        "HTTP/4.5",
                    ),
                    Code(
                        200,
                    ),
                ),
            ),
            errs: [
                Unknown http version at 0..8,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_recovers_invalid_status() {
        let test_str = "HTTP/3 invalidtext";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http3,
                    Invalid,
                ),
            ),
            errs: [
                invalid status: not a number at 7..18,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_recovers_invalid_large_status() {
        let test_str = "HTTP/3 18446744073709551616";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http3,
                    Invalid,
                ),
            ),
            errs: [
                invalid status: too large at 7..27,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_any_http_status() {
        let test_str = "HTTP/1.1 *";
        assert_debug_snapshot!(
        http_status_line_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Http1_1,
                    Any,
                ),
            ),
            errs: [],
        }
        ",
        );
    }
}

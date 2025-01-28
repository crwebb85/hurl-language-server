use crate::parser::expr::variable_name_parser;
use crate::parser::primitives::escaped_unicode_parser;

use super::filename::filename_parser;
use super::key_value::value_parser;
use super::primitives::{lt_parser, sp_parser};
use super::template::template_parser;
use super::types::{
    BooleanOption, Duration, DurationOption, DurationUnit, IntegerOption, InterpolatedString,
    InterpolatedStringPart, RequestOption, VariableDefinitionOption,
};
use super::variable::variable_value_parser;
use chumsky::prelude::*;

fn integer_option_parser<'a>(
    option_identifier: &'a str,
) -> impl Parser<'a, &'a str, IntegerOption, extra::Err<Rich<'a, char>>> + Clone {
    let integer_option = choice((
        text::int(10).to_slice().validate(|v: &str, e, emitter| {
            match v.parse::<u64>() {
                Ok(n) => IntegerOption::Literal(n),
                Err(_) => {
                    emitter.emit(Rich::custom(
                        e.span(),
                        format!(
                            "The integer value is larger than {} and is not valid for 64bit version of hurl",
                            u64::MAX
                        ),
                    ));
                    IntegerOption::BigInteger(v.to_string())
                }
            }
        }),
        template_parser().map(IntegerOption::Template)
    ));

    let option = just(option_identifier)
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(integer_option)
        .then_ignore(lt_parser())
        .map(|(_, o)| o);
    option.boxed()
}

fn boolean_option_parser<'a>(
    option_identifier: &'a str,
) -> impl Parser<'a, &'a str, BooleanOption, extra::Err<Rich<'a, char>>> + Clone {
    let boolean_option = choice((
        just("false").to(BooleanOption::Literal(false)),
        just("true").to(BooleanOption::Literal(true)),
        template_parser().map(|t| BooleanOption::Template(t)),
    ));

    let option = just(option_identifier)
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(boolean_option)
        .then_ignore(lt_parser())
        .map(|(_, o)| o);
    option.boxed()
}

fn duration_option_parser<'a>(
    option_identifier: &'a str,
) -> impl Parser<'a, &'a str, DurationOption, extra::Err<Rich<'a, char>>> + Clone {
    let duration_literal = text::int(10)
        .to_slice()
        .then(
            choice((
                just("ms").to(DurationUnit::Millisecond),
                just("s").to(DurationUnit::Second),
                just("m").to(DurationUnit::Minute),
            ))
            .or_not(),
        )
        .map(|(duration, unit): (&str, _)| {
            DurationOption::Literal(Duration {
                //TODO handle parsing errors
                duration: duration.parse::<u64>().unwrap(),
                unit,
            })
        });

    let duration_option = choice((
        template_parser().map(DurationOption::Template),
        duration_literal,
    ));

    let option = just(option_identifier)
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(duration_option)
        .then_ignore(lt_parser())
        .map(|(_, o)| o);
    option.boxed()
}

fn value_string_option_parser<'a>(
    option_identifier: &'a str,
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let option = just(option_identifier)
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(value_parser())
        .then_ignore(lt_parser())
        .map(|(_, o)| o);

    option.boxed()
}

fn filename_option_parser<'a>(
    option_identifier: &'a str,
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let option = just(option_identifier)
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(filename_parser())
        .then_ignore(lt_parser())
        .map(|(_, o)| o);

    option.boxed()
}

fn filename_password_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    let filename_password_string_escaped_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('#').to('#'),
            just(';').to(';'),
            just(' ').to(' '),
            just('{').to('{'),
            just('}').to('}'),
            just(':').to(':'),
        )))
        .or(escaped_unicode_parser());
    filename_password_string_escaped_char.boxed()
}

fn filename_password_option_parser<'a>(
    option_identifier: &'a str,
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let filename_password_str_part = choice((
        none_of("#;{} \n\\"),
        filename_password_string_escaped_char_parser(),
    ))
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("filename_password_str");

    let filename_password_template_part = template_parser()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("filename_password_template");

    let filename_password = choice((filename_password_str_part, filename_password_template_part))
        .repeated()
        .at_least(1)
        .collect::<Vec<InterpolatedStringPart>>()
        .map(|k| InterpolatedString { parts: k })
        .labelled("filename_password");

    let option = just(option_identifier)
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(filename_password)
        .then_ignore(lt_parser())
        .map(|(_, o)| o);

    option.boxed()
}

fn variable_option_parser<'a>(
) -> impl Parser<'a, &'a str, VariableDefinitionOption, extra::Err<Rich<'a, char>>> + Clone {
    let variable_definition = variable_name_parser()
        .then_ignore(just("=").padded_by(sp_parser().repeated()))
        .then(variable_value_parser())
        .map(|(name, value)| VariableDefinitionOption { name, value });

    let option = just("variable")
        .padded_by(sp_parser().repeated())
        .then_ignore(just(":").padded_by(sp_parser().repeated()))
        .then(variable_definition)
        .then_ignore(lt_parser())
        .map(|(_, o)| o);

    option.boxed()
}

pub fn option_parser<'a>(
) -> impl Parser<'a, &'a str, RequestOption, extra::Err<Rich<'a, char>>> + Clone {
    //TODO a tokenizer would likely make this parsing more efficient
    let boolean_request_option = choice((
        boolean_option_parser("compressed").map(RequestOption::Compressed),
        boolean_option_parser("location-trusted").map(RequestOption::LocationTrusted),
        boolean_option_parser("location").map(RequestOption::Location),
        boolean_option_parser("http1.0").map(RequestOption::Http10),
        boolean_option_parser("http1.1").map(RequestOption::Http11),
        boolean_option_parser("http2").map(RequestOption::Http2),
        boolean_option_parser("http3").map(RequestOption::Http3),
        boolean_option_parser("insecure").map(RequestOption::Insecure),
        boolean_option_parser("ipv4").map(RequestOption::Ipv4),
        boolean_option_parser("ipv6").map(RequestOption::Ipv6),
        boolean_option_parser("netrc-optional").map(RequestOption::NetrcOptional),
        boolean_option_parser("netrc").map(RequestOption::Netrc),
        boolean_option_parser("path-as-is").map(RequestOption::PathAsIs),
        boolean_option_parser("skip").map(RequestOption::Skip),
        boolean_option_parser("verbose").map(RequestOption::Verbose),
        boolean_option_parser("very-verbose").map(RequestOption::VeryVerbose),
    ));

    let duration_request_option = choice((
        duration_option_parser("connect-timeout").map(RequestOption::ConnectTimeout),
        duration_option_parser("delay").map(RequestOption::Delay),
        duration_option_parser("retry-interval").map(RequestOption::RetryInterval),
    ));

    let integer_request_option = choice((
        integer_option_parser("limit-rate").map(RequestOption::LimitRate),
        integer_option_parser("max-redirs").map(RequestOption::MaxRedirs),
        integer_option_parser("repeat").map(RequestOption::Repeat),
        integer_option_parser("retry").map(RequestOption::Retry),
    ));

    let value_string_request_option = choice((
        value_string_option_parser("aws-sigv4").map(RequestOption::AwsSigv4),
        value_string_option_parser("connect-to").map(RequestOption::ConnectTo),
        value_string_option_parser("netrc-file").map(RequestOption::NetrcFile),
        value_string_option_parser("proxy").map(RequestOption::Proxy),
        value_string_option_parser("resolve").map(RequestOption::Resolve),
        value_string_option_parser("unix-socket").map(RequestOption::UnixSocket),
        value_string_option_parser("user").map(RequestOption::User),
    ));

    let filename_request_option = choice((
        filename_option_parser("cacert").map(RequestOption::Cacert),
        //TODO offspec for key and output. The official parser parses as filenames but the spec
        //says they are string-values
        filename_option_parser("key").map(RequestOption::Key),
        filename_option_parser("output").map(RequestOption::Output),
    ));

    let filename_password_request_option =
        filename_password_option_parser("cert").map(RequestOption::Cert);

    let variable_request_option = variable_option_parser().map(RequestOption::Variable);

    let option = choice((
        boolean_request_option,
        duration_request_option,
        integer_request_option,
        value_string_request_option,
        filename_request_option,
        filename_password_request_option,
        variable_request_option,
    ))
    .labelled("option");

    option.boxed()
}

pub fn options_parser<'a>(
) -> impl Parser<'a, &'a str, Vec<RequestOption>, extra::Err<Rich<'a, char>>> + Clone {
    let options = option_parser().repeated().collect::<Vec<RequestOption>>();
    options.boxed()
}

#[cfg(test)]
mod option_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_option_variables() {
        let test_str = "variable: host=example.net\nvariable: id=1234";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_option_variable_with_extra_spaces() {
        let test_str = " variable   : host  =  example.net";
        assert_debug_snapshot!(
        option_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_boolean_options() {
        let test_str = "compressed: true\nlocation: true\nlocation-trusted: true\nhttp1.0: false\nhttp1.1: false\nhttp2: false\nhttp3: true\ninsecure: false\nipv4: false\nipv6: true\nnetrc: true\nnetrc-optional: true\npath-as-is: true\nskip: false\nverbose: true\nvery-verbose: true";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_templates() {
        let test_str =
            "connect-timeout: {{connectTimeout}}\ndelay: {{delay}}\nretry-interval: {{retryInterval}}";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_unit() {
        let test_str = "connect-timeout: 5\ndelay: 4\nretry-interval: 500";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_s_unit() {
        let test_str = "connect-timeout: 5s\ndelay: 4s\nretry-interval: 500s";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_ms_unit() {
        let test_str = "connect-timeout: 5ms\ndelay: 4ms\nretry-interval: 500ms";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    //TODO I might want to detect this and have a code action to fix it
    fn it_errors_duration_option_from_whitespace_left_padded_unit() {
        let test_str = "connect-timeout: 5 ms";
        assert_debug_snapshot!(
        option_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''m'' at 19..20 expected spacing, comment, newline, or end of input,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_duration_options_with_default_m_unit() {
        let test_str = "connect-timeout: 5m\ndelay: 4m\nretry-interval: 500m";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_retry_interval_option_with_default_unit() {
        let test_str = "retry-interval: 500";
        assert_debug_snapshot!(
        option_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
                RetryInterval(
                    Literal(
                        Duration {
                            duration: 500,
                            unit: None,
                        },
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_single_duration_options_with_default_unit() {
        let test_str = "delay: 4";
        assert_debug_snapshot!(
        option_parser().then_ignore(end()).parse(test_str),
            @r"
        ParseResult {
            output: Some(
                Delay(
                    Literal(
                        Duration {
                            duration: 4,
                            unit: None,
                        },
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_integer_options() {
        let test_str = "limit-rate: 59\nmax-redirs: 109\nrepeat: 10\nretry: 5";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_integer_options_with_templates() {
        let test_str = "limit-rate: {{limit_retry}}\nmax-redirs: {{max_retries}}\nrepeat: {{repeat}}\nretry: {{retry}}";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
                    LimitRate(
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "limit_retry",
                                    ),
                                    filters: [],
                                },
                            },
                        ),
                    ),
                    MaxRedirs(
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "max_retries",
                                    ),
                                    filters: [],
                                },
                            },
                        ),
                    ),
                    Repeat(
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "repeat",
                                    ),
                                    filters: [],
                                },
                            },
                        ),
                    ),
                    Retry(
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "retry",
                                    ),
                                    filters: [],
                                },
                            },
                        ),
                    ),
                ],
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_integer_option_with_partial_template() {
        //Integer options must be either an integer or a template. They
        //cannot be a mix of both
        let test_str = "limit-rate: 5{{magnitude}}";
        assert_debug_snapshot!(
        option_parser().then_ignore(end()).parse(&test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''{'' at 13..14 expected digit, or line terminator,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_largest_valid_integer_option_for_u64() {
        let test_str = format!("limit-rate: {}", u64::MAX,);
        assert_debug_snapshot!(
        option_parser().then_ignore(end()).parse(&test_str),
            @r"
        ParseResult {
            output: Some(
                LimitRate(
                    Literal(
                        18446744073709551615,
                    ),
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
        let test_str = "limit-rate: 18446744073709551616";
        assert_debug_snapshot!(
        option_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                LimitRate(
                    BigInteger(
                        "18446744073709551616",
                    ),
                ),
            ),
            errs: [
                The integer value is larger than 18446744073709551615 and is not valid for 64bit version of hurl at 12..32,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_largest_valid_integer_option_for_u32() {
        let test_str = format!("limit-rate: {}", u32::MAX,);
        assert_debug_snapshot!(
        option_parser().then_ignore(end()).parse(&test_str),
            @r"
        ParseResult {
            output: Some(
                LimitRate(
                    Literal(
                        4294967295,
                    ),
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_value_string_options() {
        let test_str = "aws-sigv4: aws:amz:eu-central-1:sts\nconnect-to: example.com:8000:127.0.0.1:8080\nnetrc-file: ~/.netrc\nproxy: example.proxy:8050\nresolve: example.com:8000:127.0.0.1\nunix-socket: sock\nuser: joe=secret";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_value_string_options_with_templates() {
        let test_str = "aws-sigv4: {{aws}}\nconnect-to: {{host}}:{{port}}:127.0.0.1:8080\nnetrc-file: {{filepath}}\nproxy: {{proxyhost}}:8050\nresolve: {{host}}:{{port}}:127.0.0.1\nunix-socket: {{socket}}\nuser: {{user}}={{password}}";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_filename_options() {
        let test_str = "cacert: /etc/cert.pem\nkey: .ssh/id_rsa.pub\noutput: ./myreport";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_filename_options_with_templates() {
        let test_str = "cacert: {{certfilepath}}\nkey: {{keyfilepath}}\noutput: {{reportfilepath}}";
        assert_debug_snapshot!(
        options_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                [
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
            ),
            errs: [],
        }
        "#,
        );
    }
}

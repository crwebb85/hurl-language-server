use crate::parser::expr::variable_name_parser;

use super::filename::filename_parser;
use super::key_value::{key_parser, value_parser};
use super::primitives::{lt_parser, sp_parser};
use super::quoted_string::quoted_string_parser;
use super::template::template_parser;
use super::types::{
    BooleanOption, Duration, DurationOption, DurationUnit, IntegerOption, InterpolatedString,
    InterpolatedStringPart, RequestOption, VariableDefinitionOption, VariableValue,
};
use chumsky::prelude::*;
use ordered_float::OrderedFloat;

pub fn option_parser() -> impl Parser<char, RequestOption, Error = Simple<char>> + Clone {
    let sp = sp_parser();
    let lt = lt_parser();
    let template = template_parser();
    let quoted_string = quoted_string_parser();
    let key = key_parser();
    let value = value_parser();
    let filename = filename_parser();

    let boolean_option = choice::<_, Simple<char>>([
        just("false").to(BooleanOption::Literal(false)),
        just("true").to(BooleanOption::Literal(true)),
    ])
    .or(template.clone().map(|t| BooleanOption::Template(t)));

    #[derive(Clone)]
    enum RequestBooleanOption {
        Compressed,
        Location,
        LocationTrusted,
        Http10,
        Http11,
        Http2,
        Http3,
        Insecure,
        Ipv4,
        Ipv6,
        Netrc,
        NetrcOptional,
        PathAsIs,
        Skip,
        Verbose,
        VeryVerbose,
    }

    let boolean_request_option_key_word = choice::<_, Simple<char>>([
        just("compressed").to(RequestBooleanOption::Compressed),
        just("location-trusted").to(RequestBooleanOption::LocationTrusted),
        just("location").to(RequestBooleanOption::Location),
        just("http1.0").to(RequestBooleanOption::Http10),
        just("http1.1").to(RequestBooleanOption::Http11),
        just("http2").to(RequestBooleanOption::Http2),
        just("http3").to(RequestBooleanOption::Http3),
        just("insecure").to(RequestBooleanOption::Insecure),
        just("ipv4").to(RequestBooleanOption::Ipv4),
        just("ipv6").to(RequestBooleanOption::Ipv6),
        just("netrc-optional").to(RequestBooleanOption::NetrcOptional),
        just("netrc").to(RequestBooleanOption::Netrc),
        just("path-as-is").to(RequestBooleanOption::PathAsIs),
        just("skip").to(RequestBooleanOption::Skip),
        just("verbose").to(RequestBooleanOption::Verbose),
        just("very-verbose").to(RequestBooleanOption::VeryVerbose),
    ])
    .labelled("boolean_request_option_keyword");

    let boolean_request_option = boolean_request_option_key_word
        .then_ignore(sp.clone().repeated())
        .then_ignore(just(":"))
        .then_ignore(sp.clone().repeated())
        .then(boolean_option)
        .then_ignore(lt.clone())
        .map(|(option_type, option)| match option_type {
            RequestBooleanOption::Compressed => RequestOption::Compressed(option),
            RequestBooleanOption::Location => RequestOption::Location(option),
            RequestBooleanOption::LocationTrusted => RequestOption::LocationTrusted(option),
            RequestBooleanOption::Http10 => RequestOption::Http10(option),
            RequestBooleanOption::Http11 => RequestOption::Http11(option),
            RequestBooleanOption::Http2 => RequestOption::Http2(option),
            RequestBooleanOption::Http3 => RequestOption::Http3(option),
            RequestBooleanOption::Insecure => RequestOption::Insecure(option),
            RequestBooleanOption::Ipv4 => RequestOption::Ipv4(option),
            RequestBooleanOption::Ipv6 => RequestOption::Ipv6(option),
            RequestBooleanOption::Netrc => RequestOption::Netrc(option),
            RequestBooleanOption::NetrcOptional => RequestOption::NetrcOptional(option),
            RequestBooleanOption::PathAsIs => RequestOption::PathAsIs(option),
            RequestBooleanOption::Skip => RequestOption::Skip(option),
            RequestBooleanOption::Verbose => RequestOption::Verbose(option),
            RequestBooleanOption::VeryVerbose => RequestOption::VeryVerbose(option),
        })
        .labelled("request_boolean_option");

    #[derive(Clone)]
    enum RequestDurationOption {
        ConnectTimeout,
        Delay,
        RetryInterval,
    }

    let duration_literal = text::int(10)
        .then(
            choice::<_, Simple<char>>([
                just("ms").to(DurationUnit::Millisecond),
                just("s").to(DurationUnit::Second),
                just("m").to(DurationUnit::Minute),
            ])
            .or_not(),
        )
        .map(|(duration, unit)| {
            DurationOption::Literal(Duration {
                //TODO handle parsing errors
                duration: duration.parse::<u64>().unwrap(),
                unit,
            })
        })
        .labelled("duration_literal");

    let duration_option = template
        .clone()
        .map(DurationOption::Template)
        .or(duration_literal)
        .labelled("duration_option");

    let duration_request_option = choice::<_, Simple<char>>([
        just("connect-timeout").to(RequestDurationOption::ConnectTimeout),
        just("delay").to(RequestDurationOption::Delay),
        just("retry-interval").to(RequestDurationOption::RetryInterval),
    ])
    .then_ignore(sp.clone().repeated())
    .then_ignore(just(":"))
    .then_ignore(sp.clone().repeated())
    .then(duration_option)
    .then_ignore(lt.clone())
    .map(|(option_type, option)| match option_type {
        RequestDurationOption::ConnectTimeout => RequestOption::ConnectTimeout(option),
        RequestDurationOption::Delay => RequestOption::Delay(option),
        RequestDurationOption::RetryInterval => RequestOption::RetryInterval(option),
    });

    #[derive(Clone)]
    enum RequestIntegerOption {
        LimitRate,
        MaxRedirs,
        Repeat,
        Retry,
    }

    let integer_option = text::int(10).map(|v: String| match v.parse::<usize>() {
        Ok(n) => IntegerOption::Literal(n),
        Err(_) => IntegerOption::BigInteger(v),
    });

    let integer_request_option = choice::<_, Simple<char>>([
        just("limit-rate").to(RequestIntegerOption::LimitRate),
        just("max-redirs").to(RequestIntegerOption::MaxRedirs),
        just("repeat").to(RequestIntegerOption::Repeat),
        just("retry").to(RequestIntegerOption::Retry),
    ])
    .then_ignore(sp.clone().repeated())
    .then_ignore(just(":"))
    .then_ignore(sp.clone().repeated())
    .then(integer_option)
    .then_ignore(lt.clone())
    .map(|(option_type, option)| match option_type {
        RequestIntegerOption::LimitRate => RequestOption::LimitRate(option),
        RequestIntegerOption::MaxRedirs => RequestOption::MaxRedirs(option),
        RequestIntegerOption::Repeat => RequestOption::Repeat(option),
        RequestIntegerOption::Retry => RequestOption::Retry(option),
    });

    #[derive(Clone)]
    enum RequestValueStringOption {
        AwsSigv4,
        ConnectTo,
        NetrcFile,
        Proxy,
        Resolve,
        UnixSocket,
        User,
    }

    let value_string_option = value.clone();

    let value_string_request_option = choice::<_, Simple<char>>([
        just("aws-sigv4").to(RequestValueStringOption::AwsSigv4),
        just("connect-to").to(RequestValueStringOption::ConnectTo),
        just("netrc-file").to(RequestValueStringOption::NetrcFile),
        just("proxy").to(RequestValueStringOption::Proxy),
        just("resolve").to(RequestValueStringOption::Resolve),
        just("unix-socket").to(RequestValueStringOption::UnixSocket),
        just("user").to(RequestValueStringOption::User),
    ])
    .then_ignore(sp.clone().repeated())
    .then_ignore(just(":"))
    .then_ignore(sp.clone().repeated())
    .then(value_string_option)
    .then_ignore(lt.clone())
    .map(|(option_type, option)| match option_type {
        RequestValueStringOption::AwsSigv4 => RequestOption::AwsSigv4(option),
        RequestValueStringOption::ConnectTo => RequestOption::ConnectTo(option),
        RequestValueStringOption::NetrcFile => RequestOption::NetrcFile(option),
        RequestValueStringOption::Proxy => RequestOption::Proxy(option),
        RequestValueStringOption::Resolve => RequestOption::Resolve(option),
        RequestValueStringOption::UnixSocket => RequestOption::UnixSocket(option),
        RequestValueStringOption::User => RequestOption::User(option),
    });

    #[derive(Clone)]
    enum RequestFilenameOption {
        Cacert,
        Key,
        Output,
    }

    let filename_request_option = choice::<_, Simple<char>>([
        just("cacert").to(RequestFilenameOption::Cacert),
        //TODO offspec for key and output. The official parser parses as filenames but the spec
        //says they are string-values
        just("key").to(RequestFilenameOption::Key),
        just("output").to(RequestFilenameOption::Output),
    ])
    .then_ignore(sp.clone().repeated())
    .then_ignore(just(":"))
    .then_ignore(sp.clone().repeated())
    .then(filename)
    .then_ignore(lt.clone())
    .map(|(option_type, option)| match option_type {
        RequestFilenameOption::Cacert => RequestOption::Cacert(option),
        RequestFilenameOption::Key => RequestOption::Key(option),
        RequestFilenameOption::Output => RequestOption::Output(option),
    });

    let filename_password_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .or(just('\\').to('\\'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t'))
                .or(just('#').to('#'))
                .or(just(';').to(';'))
                .or(just(' ').to(' '))
                .or(just('{').to('{'))
                .or(just('}').to('}'))
                .or(just(':').to(':'))
                .or(just('u').ignore_then(
                    filter(|c: &char| c.is_digit(16))
                        .repeated()
                        .exactly(4)
                        .collect::<String>()
                        .validate(|digits, span, emit| {
                            char::from_u32(u32::from_str_radix(&digits, 16).unwrap())
                                .unwrap_or_else(|| {
                                    emit(Simple::custom(span, "invalid unicode character"));
                                    '\u{FFFD}' // unicode replacement character
                                })
                        }),
                )),
        )
        .labelled("filename_password_string_escaped_char");

    let filename_password_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        // ~[#;{} \n\\]+
        c != &'#' && c != &';' && c != &'{' && c != &'}' && c != &' ' && c != &'\n' && c != &'\\'
    })
    .or(filename_password_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("filename_password_str");

    let filename_password_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("filename_password_template");

    let filename_password = filename_password_str_part
        .or(filename_password_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k })
        .labelled("filename_password");

    let filename_password_request_option = just("cert")
        .then_ignore(sp.clone().repeated())
        .then_ignore(just(":"))
        .then_ignore(sp.clone().repeated())
        .then(filename_password)
        .then_ignore(lt.clone())
        .map(|(_, filename_password)| RequestOption::Cert(filename_password));

    let float = text::int(10)
        .then_ignore(just('.'))
        .then(text::digits(10))
        .map(|(integer_part, fraction_part)| {
            let value: f64 = format!("{}.{}", integer_part, fraction_part)
                .parse()
                .unwrap();

            VariableValue::Float(OrderedFloat::<f64>::from(value))
        });

    let variable_value = choice([
        just("null").to(VariableValue::Null),
        just("true").to(VariableValue::Boolean(true)),
        just("false").to(VariableValue::Boolean(false)),
        just("false").to(VariableValue::Boolean(false)),
    ])
    .or(text::int(10)
        .from_str::<i64>()
        .unwrapped()
        .map(VariableValue::Integer))
    .or(float)
    .or(key.map(VariableValue::String))
    .or(quoted_string.map(VariableValue::String));

    let variable_name = variable_name_parser();

    let variable_definition = variable_name
        .clone()
        .then_ignore(sp.clone().repeated())
        .then_ignore(just("="))
        .then_ignore(sp.clone().repeated())
        .then(variable_value)
        .map(|(name, value)| VariableDefinitionOption { name, value });

    let variable_request_option = just("variable")
        .then_ignore(sp.clone().repeated())
        .then_ignore(just(":"))
        .then_ignore(sp.clone().repeated())
        .then(variable_definition)
        .then_ignore(lt.clone())
        .map(|(_, variable_definition)| RequestOption::Variable(variable_definition));

    let option = boolean_request_option
        .or(duration_request_option)
        .or(integer_request_option)
        .or(value_string_request_option)
        .or(filename_request_option)
        .or(filename_password_request_option)
        .or(variable_request_option);

    option
}

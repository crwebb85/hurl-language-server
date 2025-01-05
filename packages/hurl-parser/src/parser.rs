pub mod types;
use chumsky::prelude::*;
use ordered_float::OrderedFloat;
use text::TextParser;
use types::{
    BasicAuthSection, BooleanOption, CookiesSection, Duration, DurationOption, DurationUnit, Entry, Expr, FileKeyValue, FileValue, FilterFunction, FormParamsSection, IntegerOption, InterpolatedString, InterpolatedStringPart, KeyValue, Method, MultipartFormDataSection, MultipartFormParam, QueryStringParamsSection, Request, RequestOption, RequestOptionsSection, RequestSection, Template, ExprValue, VariableDefinitionOption, VariableValue
};
#[cfg(test)]
mod test;

pub fn ast_parser() -> impl Parser<char, Vec<Entry>, Error = Simple<char>> {
    let method = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|value| Method { value })
        .padded();

    let sp = filter(|c: &char| c.is_whitespace() && (c == &'\t' || c == &' '));

    let comment = just('#').then(
        filter::<_, _, Simple<char>>(|c| c != &'\n')
            .repeated()
            .at_least(1),
    );

    let lt = sp
        .clone()
        .repeated()
        .then(comment)
        .or_not() // or_not makes the comment optional
        .then(text::newline().or(end()));

    let expr_function = choice::<_, Simple<char>>([
        text::keyword("getEnv").to(ExprValue::FunctionName("getEnv".to_owned())), 
        text::keyword("newDate").to(ExprValue::FunctionName("newDate".to_owned())),
        text::keyword("newUuid").to(ExprValue::FunctionName("newUuid".to_owned()))
    ]);

    let variable_name = 
        filter::<_, _, Simple<char>>(char::is_ascii_alphanumeric)
            .repeated()
            .at_least(1).collect::<String>();

    let expr_variable = variable_name.map(ExprValue::VariableName);

    let quoted_string_escaped_char = just('\\').ignore_then(
        just('\\')
            .or(just('\\').to('\\'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(|c: &char| c.is_digit(16))
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    ).labelled("quoted_string_escaped_char");

    let quoted_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c != &'"' && c != &'\\' 
    })
    .or(quoted_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);

    let expr_variable = expr_function.or(expr_variable.clone());

    let template = recursive(|template| {

        let quoted_template_part = template
            .map(|t| InterpolatedStringPart::Template(t));

        let quoted_part = quoted_template_part
            .or(quoted_str_part);

        let quoted_string = just("\"")
            .ignored()
            .then(quoted_part.repeated().at_least(1))
            .then_ignore(just("\""))
            .map(|(_, v)| InterpolatedString { parts: v }).labelled("quoted_string");

        let decode_filter_function = just("decode")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::Decode { encoding: s});

        let format_filter_function = just("format")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::Format { fmt: s});

        let jsonpath_filter_function = just("jsonpath")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::JsonPath { expr: s });

        let nth_filter_function = just("nth")
            .then_ignore(sp.clone())
            .then(text::int(10))
            .map(|(_, n)| FilterFunction::Nth { 
                nth: n.parse::<u64>()
                    .expect("TODO implement error recovery for invalid integers used in the Nth filter function argument") 
            });

        let regex_filter_function = just("regex")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::Regex { value: s });

        let split_filter_function = just("split")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::Split { sep: s });
            
        let replace_filter_function = just("replace")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|((_, old), new)| FilterFunction::Replace { old_value: old, new_value: new });

        let todate_filter_function = just("toDate")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::ToDate { fmt: s });

        let xpath_filter_function = just("xpath")
            .then_ignore(sp.clone())
            .then(quoted_string.clone())
            .map(|(_, s)| FilterFunction::XPath { expr: s });

        let filter_function = choice::<_, Simple<char>>([
            just("count").to(FilterFunction::Count), 
            just("daysAfterNow").to(FilterFunction::DaysAfterNow),
            just("daysBeforeNow").to(FilterFunction::DaysBeforeNow),
            just("htmlEscape").to(FilterFunction::HtmlEscape),
            just("htmlUnescape").to(FilterFunction::HtmlUnescape),
            just("toFloat").to(FilterFunction::ToFloat),
            just("toInt").to(FilterFunction::ToInt),
            just("urlDecode").to(FilterFunction::UrlDecode),
            just("urlEncode").to(FilterFunction::UrlEncode),
        ])
            .or(decode_filter_function)
            .or(format_filter_function)
            .or(jsonpath_filter_function)
            .or(nth_filter_function)
            .or(regex_filter_function)
            .or(split_filter_function)
            .or(replace_filter_function)
            .or(todate_filter_function)
            .or(xpath_filter_function);

            let expr = expr_variable
            .then_ignore(sp.clone().or_not())
            .then(filter_function.separated_by(sp.clone()))
            .map( |(expr_var, filter_funcs)| Expr {
                variable: expr_var,
                filters: filter_funcs
            });

        just("{")
        .ignored()
        .then_ignore(just("{"))
        .then(expr)
        .then_ignore(just("}"))
        .then_ignore(just("}"))
        .map(|(_, captured_expr)| Template {
            expr: captured_expr,
        })
    }).labelled("template"); 


    let quoted_template_part = template.clone()
        .map(|t| InterpolatedStringPart::Template(t));

    let quoted_part = quoted_template_part
        .or(quoted_str_part);

    let quoted_string = just("\"")
        .ignored()
        .then(quoted_part.repeated().at_least(1))
        .then_ignore(just("\""))
        .map(|(_, v)| InterpolatedString { parts: v }).labelled("quoted_string");

    let key_string_escaped_char = just('\\').ignore_then(
        //TODO for some reason when I test hurl files with the hurl cli using
        //these escape sequences I get errors. I need to investivate if that is
        //a version issue or if my understanding on this grammar is wrong
        just('\\')
            .or(just('#').to('#'))
            .or(just(':').to(':'))
            .or(just('\\').to('\\'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
         .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(|c: &char| c.is_digit(16))
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    ).labelled("key_string_escaped_char");

    let key_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric()
            || c == &'_'
            || c == &'-'
            || c == &'.'
            || c == &'['
            || c == &']'
            || c == &'@'
            || c == &'$'
    })
    .or(key_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str).labelled("key_str");

    let key_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t)).labelled("key_template");

    let key = key_str_part
        .or(key_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k }).labelled("key");

    let value_string_escaped_char = just('\\').ignore_then(
        just('\\')
            .or(just('#').to('#'))
            .or(just('\\').to('\\'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(|c: &char| c.is_digit(16))
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    ).labelled("value_escaped_char");

    let value_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c != &'#' && c != &'\n' && c != &'\\' 
            //currly brackets while allowed will be handled after trying to parse
            //as a template
            && c != &'{' && c != &'}'
    })
    .or(value_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str).labelled("value_str");


    let value_brackets = filter::<_, _, Simple<char>>(|c: &char| {
        c == &'{' || c == &'}'
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str).labelled("value_brackets");

    let value_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t)).labelled("value_template");

    let value_part = value_template_part
        .or(value_str_part)
        .or(value_brackets);

    let value = value_part
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v }).labelled("value");

    let key_value = key.clone()
        .then_ignore(just(':'))
        .then_ignore(sp.clone().repeated())//TODO: I think this is an offspec sp
        .then(value.clone())
        .map(|(key, value)| KeyValue { key, value }).labelled("key_value");

    let key_values = key_value.clone().then_ignore(lt.clone()).repeated();

    let headers = key_values.clone(); 

    let basic_auth_section =  just("[BasicAuth]")
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map( |(_, auth_key_values)| RequestSection::BasicAuthSection(BasicAuthSection {
        key_values: auth_key_values
    }));

    let query_string_params_section =  just("[QueryStringParams]").or(just("[Query]"))
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map( |(_, query_key_values)| RequestSection::QueryStringParamsSection(QueryStringParamsSection {
        queries: query_key_values
    }));

    let form_params_section =  just("[FormParams]").or(just("[Form]"))
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map( |(_, form_params)| RequestSection::FormParamsSection(FormParamsSection {
        params: form_params
    }));


    let filename_string_escaped_char = just('\\').ignore_then(
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
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    ).labelled("filename_string_escaped_char");

    let filename_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        // ~[#;{} \n\\]+
            c != &'#'
            && c != &';'
            && c != &'{'
            && c != &'}'
            && c != &' '
            && c != &'\n'
            && c != &'\\'
    })
    .or(filename_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str).labelled("filename_str");

    let file_content_type = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric()
            || c == &'/'
            || c == &'+'
            || c == &'-'
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .labelled("file_content_type");

    let filename_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t)).labelled("filename_template");

    let filename = filename_str_part
        .or(filename_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k }).labelled("filename");

    let file_value = just("file,")
        .then(filename.clone())
        .then_ignore(just(';'))
        .then(file_content_type.or_not())
        .map(|((_, filename), content_type)| FileValue {
            filename,
            content_type
            }
        );

    let file_param = key.clone()
        .then_ignore(sp.clone().repeated())//TODO: I think this is an offspec sp
        .then_ignore(just(':'))
        .then_ignore(sp.clone().repeated())//TODO: I think this is an offspec sp
        .then(file_value)
        .map(|(key, value)| MultipartFormParam::FileParam(FileKeyValue { key, value })).labelled("file_key_value");

    let multipart_form_param = file_param.or(key_value.map(MultipartFormParam::KeyValueParam));

    let multipart_form_data_section =  just("[MultipartFormData]").or(just("[Multipart]"))
        .then_ignore(lt.clone())
        .then(multipart_form_param.repeated()) 
        .map( |(_, file_params)| RequestSection::MultipartFormDataSection(MultipartFormDataSection {
        params: file_params
    }));

    let cookies_section =  just("[Cookies]")
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map( |(_, cookies_key_value)| RequestSection::CookiesSection(CookiesSection {
            cookies: cookies_key_value,
    }));


    let boolean_option = choice::<_, Simple<char>>([
        just("false").to(BooleanOption::Literal(false)), 
        just("true").to(BooleanOption::Literal(true)), 
    ]).or(template.clone().map(|t| BooleanOption::Template(t)));

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
    ]).labelled("boolean_request_option_keyword");

    let boolean_request_option = 
    boolean_request_option_key_word
    .then_ignore(sp.clone().repeated())
    .then_ignore(just(":"))
    .then_ignore(sp.clone().repeated())
    .then(boolean_option)
    .then_ignore(lt.clone())
    .map(|(option_type, option)| {
        match option_type {
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
        }
    }).labelled("request_boolean_option");

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
                ]).or_not()
            )
            .map(|(duration, unit)| DurationOption::Literal(Duration { 
                //TODO handle parsing errors
                duration: duration.parse::<u64>().unwrap(), 
                unit 
            })).labelled("duration_literal");

    let duration_option = template.clone()
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
    .map(|(option_type, option)| {
        match option_type {
            RequestDurationOption::ConnectTimeout => RequestOption::ConnectTimeout(option),
            RequestDurationOption::Delay => RequestOption::Delay(option),
            RequestDurationOption::RetryInterval => RequestOption::RetryInterval(option),
        }
    });

    #[derive(Clone)]
    enum RequestIntegerOption {
        LimitRate,
        MaxRedirs,
        Repeat,
        Retry,
    }

    let integer_option = text::int(10).map(|v:String| IntegerOption::Literal(v.parse::<usize>().unwrap()));

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
    .map(|(option_type, option)| {
        match option_type {
            RequestIntegerOption::LimitRate => RequestOption::LimitRate(option), 
            RequestIntegerOption::MaxRedirs => RequestOption::MaxRedirs(option), 
            RequestIntegerOption::Repeat => RequestOption::Repeat(option), 
            RequestIntegerOption::Retry => RequestOption::Retry(option), 
        }
    });

    #[derive(Clone)]
    enum RequestValueStringOption {
        AwsSigv4,
        Key,
        ConnectTo,
        NetrcFile,
        Output,
        Proxy,
        Resolve,
        UnixSocket,
        User,
    }

    let value_string_option = value.clone();

    let value_string_request_option = choice::<_, Simple<char>>([
        just("aws-sigv4").to(RequestValueStringOption::AwsSigv4),
        just("key").to(RequestValueStringOption::Key),
        just("connect-to").to(RequestValueStringOption::ConnectTo),
        just("netrc-file").to(RequestValueStringOption::NetrcFile),
        just("output").to(RequestValueStringOption::Output),
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
    .map(|(option_type, option)| {
        match option_type {
            RequestValueStringOption::AwsSigv4 => RequestOption::AwsSigv4(option),
            RequestValueStringOption::Key => RequestOption::Key(option),
            RequestValueStringOption::ConnectTo => RequestOption::ConnectTo(option),
            RequestValueStringOption::NetrcFile => RequestOption::NetrcFile(option),
            RequestValueStringOption::Output => RequestOption::Output(option),
            RequestValueStringOption::Proxy => RequestOption::Proxy(option),
            RequestValueStringOption::Resolve => RequestOption::Resolve(option),
            RequestValueStringOption::UnixSocket => RequestOption::UnixSocket(option),
            RequestValueStringOption::User => RequestOption::User(option),
        }
    });

    let filename_request_option = just("cacert")
        .then_ignore(sp.clone().repeated())
        .then_ignore(just(":"))
        .then_ignore(sp.clone().repeated())
        .then(filename)
        .then_ignore(lt.clone())
        .map(|(_, filename)| RequestOption::Cacert(filename));
    
    let filename_password_string_escaped_char = just('\\').ignore_then(
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
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    ).labelled("filename_password_string_escaped_char");

    let filename_password_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        // ~[#;{} \n\\]+
            c != &'#'
            && c != &';'
            && c != &'{'
            && c != &'}'
            && c != &' '
            && c != &'\n'
            && c != &'\\'
    })
    .or(filename_password_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str).labelled("filename_password_str");

    let filename_password_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t)).labelled("filename_password_template");

    let filename_password = filename_password_str_part
        .or(filename_password_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k }).labelled("filename_password");


    let filename_password_request_option = just("cert")
        .then_ignore(sp.clone().repeated())
        .then_ignore(just(":"))
        .then_ignore(sp.clone().repeated())
        .then(filename_password)
        .then_ignore(lt.clone())
        .map(|(_, filename_password)| RequestOption::Cert(filename_password));


    let float = 
        text::int(10)
        .then_ignore(just('.'))
        .then(text::digits(10))
        .map(|(integer_part, fraction_part)| {
            let value:f64 = format!("{}.{}", integer_part, fraction_part).parse().unwrap();

            VariableValue::Float(OrderedFloat::<f64>::from(value))
        });

    let variable_value = choice([
        just("null").to(VariableValue::Null),
        just("true").to(VariableValue::Boolean(true)),
        just("false").to(VariableValue::Boolean(false)),
        just("false").to(VariableValue::Boolean(false)),
    ])
    .or(text::int(10).from_str::<i64>().unwrapped().map(VariableValue::Integer))
    .or(float)
    .or(key.map(VariableValue::String))
    .or(quoted_string.map(VariableValue::String)); 

    let variable_definition = variable_name.clone()
        .then_ignore(sp.clone().repeated())
        .then_ignore(just("="))
        .then_ignore(sp.clone().repeated())
        .then(variable_value)
        .map(|(name, value)| VariableDefinitionOption {
            name, value
        });

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
    
    let options_section =  just("[Options]")
        .then_ignore(lt.clone())
        .then(option.repeated().clone())
        .map( |(_, options)| RequestSection::OptionsSection(RequestOptionsSection {
            options,
    }));

    let request_section =
        basic_auth_section
        .or(query_string_params_section)
        .or(form_params_section)
        .or(multipart_form_data_section)
        .or(cookies_section)
        .or(options_section);
        // TODO and an unknown section for error handling
        // .or(unknown_section);

    let request = sp.clone()
        .repeated()
        .ignore_then(method
            .then(value.clone())
            .then_ignore(lt.clone())
            .then(headers)
            .then(request_section.repeated()) //TODO
            .map( |(((method_value, url_value_string), headers), request_sections)| Request {
                method: method_value,
                url: url_value_string,
                header: headers,
                request_sections
            })
        ).labelled("request");

    let entry = request.map(|request_value| Entry {
        request: Box::new(request_value),
        response: None,
    }).labelled("entry");

    entry.repeated().then_ignore(lt.clone())
}

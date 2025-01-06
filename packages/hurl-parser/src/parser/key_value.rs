use chumsky::prelude::*;
use super::{primitives::sp_parser, template::template_parser};
use super::types::{
    InterpolatedString, InterpolatedStringPart, KeyValue
};


pub fn key_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
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

    let template = template_parser();
    let key_template_part = template
        .map(|t| InterpolatedStringPart::Template(t)).labelled("key_template");

    let key = key_str_part
        .or(key_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k }).labelled("key");
    
    key
}


pub fn value_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {

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

    let template = template_parser();
    let value_template_part = template
        .map(|t| InterpolatedStringPart::Template(t)).labelled("value_template");

    let value_part = value_template_part
        .or(value_str_part)
        .or(value_brackets);

    let value = value_part
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v }).labelled("value");

    value
}

pub fn key_value_parser() -> impl Parser<char, KeyValue, Error = Simple<char>> + Clone {

    let sp = sp_parser();
    let key = key_parser();
    let value = value_parser();

    let key_value = key
        .then_ignore(just(':'))
        .then_ignore(sp.repeated())//TODO: I think this is an offspec sp
        .then(value)
        .map(|(key, value)| KeyValue { key, value }).labelled("key_value");

    key_value
}

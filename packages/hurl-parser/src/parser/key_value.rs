use super::types::{InterpolatedString, InterpolatedStringPart, KeyValue};
use super::{primitives::sp_parser, template::template_parser};
use chumsky::prelude::*;

pub fn key_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    //TODO for some reason when I test hurl files with the hurl cli using
    //these escape sequences I get errors. I need to investivate if that is
    //a version issue or if my understanding on this grammar is wrong
    let key_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .to('\\')
                .or(just('#').to('#'))
                .or(just(':').to(':'))
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
                            char::from_u32(u32::from_str_radix(&digits, 16).unwrap())
                                .unwrap_or_else(|| {
                                    emit(Simple::custom(span, "invalid unicode character"));
                                    '\u{FFFD}' // unicode replacement character
                                })
                        }),
                )),
        )
        .labelled("key-string-escaped-char");

    let key_string_text = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric()
            || c == &'_'
            || c == &'-'
            || c == &'.'
            || c == &'['
            || c == &']'
            || c == &'@'
            || c == &'$'
    })
    .labelled("key-string-text");

    let key_string_content = key_string_text
        .or(key_string_escaped_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("key-string-content");

    let template = template_parser();
    let key_template_part = template
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("key-template");

    let key_string = key_string_content
        .or(key_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k })
        .labelled("key-string");

    key_string
}

pub fn value_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    let value_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .to('\\')
                .or(just('#').to('#'))
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
                            char::from_u32(u32::from_str_radix(&digits, 16).unwrap())
                                .unwrap_or_else(|| {
                                    emit(Simple::custom(span, "invalid unicode character"));
                                    '\u{FFFD}' // unicode replacement character
                                })
                        }),
                )),
        )
        .labelled("value-string-escaped-char");

    let template = template_parser();

    let value_string_text =
        filter::<_, _, Simple<char>>(|c: &char| c != &'#' && c != &'\n' && c != &'\\' && c != &'{')
            .labelled("value-string-text");

    let value_string_content = value_string_text
        .or(value_string_escaped_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("value-string-content");

    let value_brackets = filter::<_, _, Simple<char>>(|c: &char| c == &'{')
        .repeated()
        .at_least(1)
        //Consume 2 since if those two brackets made up a template they would have
        //already been consumed
        .at_most(2)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("value_brackets");

    let value_template_part = template
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("value-template");

    //TODO normalize so that non-template brackets get combined with value_string_content
    let value_string = value_template_part
        .or(value_string_content)
        .or(value_brackets)
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v })
        .labelled("value-string");

    value_string
}

pub fn key_value_parser() -> impl Parser<char, KeyValue, Error = Simple<char>> + Clone {
    let sp = sp_parser();
    let key = key_parser();
    let value = value_parser();

    let key_value = key
        .then_ignore(sp.clone().repeated()) //TODO: I think this is an offspec sp
        .then_ignore(just(':'))
        .then_ignore(sp.repeated()) //TODO: I think this is an offspec sp
        .then(value)
        .map(|(key, value)| KeyValue { key, value })
        .labelled("key-value");

    key_value
}

pub mod types;
use chumsky::prelude::*;
use text::TextParser;
use types::{
    Entry, InterpolatedString, InterpolatedStringPart, KeyValue, Method, Request, Template,
    ValueString,
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

    let sp = choice::<_, Simple<char>>([text::keyword(" "), text::keyword("\t")]);

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

    let url = take_until(lt.clone())
        .map(|(url_chars, _)| url_chars)
        .collect::<String>()
        .map(|url_string| ValueString { value: url_string });

    //TODO finish expr parsing logic
    let expr = just(' ');

    let template = just("{{")
        .ignored()
        .then(expr)
        .then_ignore(just("}}"))
        .to(Template {}); //TODO add template contents

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
    );

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
    .map(InterpolatedStringPart::Str);

    let key_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t));

    let key = key_str_part
        .or(key_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k });

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
    );

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
    .map(InterpolatedStringPart::Str);


    let value_brackets = filter::<_, _, Simple<char>>(|c: &char| {
        c == &'{' || c == &'}'
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);

    let value_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t));

    let value = value_template_part
        .or(value_str_part)
        .or(value_brackets)
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v });

    let key_value = key
        .then_ignore(just(':'))
        .then(value)
        .map(|(key, value)| KeyValue { key, value });

    let header = key_value.repeated();

    let request = sp.repeated().ignore_then(method.then(url).then(header).map(
        |((method_value, url_value_string), headers)| Request {
            method: method_value,
            url: url_value_string,
            header: headers,
        },
    ));
    let entry = request.map(|request_value| Entry {
        request: Box::new(request_value),
        response: None,
    });
    entry.repeated().then_ignore(lt.clone())
}

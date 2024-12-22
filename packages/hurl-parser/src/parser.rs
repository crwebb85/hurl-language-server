pub mod types;
use chumsky::prelude::*;
use text::TextParser;
use types::{Entry, KeyString, KeyStringPart, KeyValue, Method, Request, ValueString};
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

    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
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

    let key = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric()
            || c == &'_'
            || c == &'-'
            || c == &'.'
            || c == &'['
            || c == &']'
            || c == &'@'
            || c == &'$'
    })
    .or(escape)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(KeyStringPart::Str)
    .map(|k| KeyString { parts: vec![k] });

    let value = take_until(lt.clone())
        .map(|(key_value, _)| key_value)
        .collect::<String>();

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

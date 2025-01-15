use super::key_value::{key_value_parser, value_parser};
use super::primitives::{lt_parser, sp_parser};
use super::request_section::request_section_parser;
use super::types::{Entry, Method, Request};
use chumsky::prelude::*;

pub fn ast_parser() -> impl Parser<char, Vec<Entry>, Error = Simple<char>> {
    let method = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|value| Method { value })
        .padded();

    let sp = sp_parser();
    let lt = lt_parser();
    let key_value = key_value_parser();
    let value = value_parser();
    let request_section = request_section_parser();

    let key_values = key_value.clone().then_ignore(lt.clone()).repeated();

    let headers = key_values.clone();

    let request = sp
        .clone()
        .repeated()
        .ignore_then(
            method
                .then(value.clone())
                .then_ignore(lt.clone())
                .then(headers)
                .then(request_section.repeated())
                .map(
                    |(((method_value, url_value_string), headers), request_sections)| Request {
                        method: method_value,
                        url: url_value_string,
                        header: headers,
                        request_sections,
                    },
                ),
        )
        .labelled("request");

    let entry = request
        .map(|request_value| Entry {
            request: Box::new(request_value),
            response: None,
        })
        .labelled("entry");

    entry.repeated().then_ignore(lt.clone())
}

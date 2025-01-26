use super::body::body_parser;
use super::key_value::{key_value_parser, value_parser};
use super::primitives::{lt_parser, sp_parser};
use super::request_section::request_section_parser;
use super::types::{Entry, KeyValue, Method, Request, RequestSection};
use chumsky::prelude::*;

pub fn ast_parser<'a>() -> impl Parser<'a, &'a str, Vec<Entry>, extra::Err<Rich<'a, char>>> + Clone
{
    let method = any()
        .filter(char::is_ascii_uppercase)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|value| Method { value })
        .padded();

    let lt = lt_parser();

    let header_line = key_value_parser().then_ignore(lt.clone());

    let request = sp_parser()
        .repeated()
        .ignore_then(method)
        .then(value_parser())
        .then_ignore(lt.clone())
        .then(header_line.repeated().collect::<Vec<KeyValue>>())
        .then(
            request_section_parser()
                .repeated()
                .collect::<Vec<RequestSection>>(),
        )
        .then(body_parser().or_not())
        .map(
            |((((method_value, url_value_string), headers), request_sections), body)| Request {
                method: method_value,
                url: url_value_string,
                header: headers,
                request_sections,
                body,
            },
        )
        .labelled("request");

    let entry = request
        .map(|request_value| Entry {
            request: Box::new(request_value),
            response: None,
        })
        .labelled("entry");

    entry
        .repeated()
        .collect::<Vec<Entry>>()
        .then_ignore(lt.clone().or_not()) //TODO fix so that any number of line terminators can
                                          //follow the entry
}

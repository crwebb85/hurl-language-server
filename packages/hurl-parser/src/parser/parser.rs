use super::body::body_parser;
use super::key_value::{key_value_parser, value_parser};
use super::primitives::{lt_parser, sp_parser};
use super::request_section::request_section_parser;
use super::types::{Ast, Entry, KeyValue, Method, Request, RequestSection};
use chumsky::prelude::*;

fn method_parser<'a>() -> impl Parser<'a, &'a str, Method, extra::Err<Rich<'a, char>>> + Clone {
    let method = text::ident()
        .and_is(
            any()
                .filter(char::is_ascii_uppercase)
                .repeated()
                .at_least(1),
        )
        .to_slice()
        .map(|value: &str| Method {
            value: value.to_string(),
        })
        .padded();
    method.boxed()
}

fn request_parser<'a>() -> impl Parser<'a, &'a str, Request, extra::Err<Rich<'a, char>>> + Clone {
    let header_line = key_value_parser().then_ignore(lt_parser());
    let request = method_parser()
        .padded_by(sp_parser().repeated())
        .then(value_parser())
        .then_ignore(lt_parser())
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
    request.boxed()
}

pub fn ast_parser<'a>() -> impl Parser<'a, &'a str, Ast, extra::Err<Rich<'a, char>>> + Clone {
    let entry = request_parser()
        .map(|request_value| Entry {
            request: Box::new(request_value),
            response: None,
        })
        .labelled("entry");

    lt_parser()
        .or_not()
        .ignore_then(entry.repeated().collect::<Vec<Entry>>())
        .map(|entries| Ast { entries })
        .boxed()
}

pub fn parse_ast<'a>(document: &'a str) -> (Option<Ast>, Vec<Rich<'a, char>>) {
    let (ast, errs) = ast_parser().parse(document).into_output_errors();
    (ast, errs)
}

use super::key_value::key_value_parser;
use super::primitives::lt_parser;
use super::types::KeyValue;
use chumsky::prelude::*;

fn header_line_parser<'a>() -> impl Parser<'a, &'a str, KeyValue, extra::Err<Rich<'a, char>>> + Clone
{
    let header_line = key_value_parser().then_ignore(lt_parser());
    header_line
}

pub fn headers_parser<'a>(
) -> impl Parser<'a, &'a str, Vec<KeyValue>, extra::Err<Rich<'a, char>>> + Clone {
    header_line_parser().repeated().collect::<Vec<KeyValue>>()
}

//TODO write tests

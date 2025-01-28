use chumsky::prelude::*;

use super::{
    primitives::todo_parser,
    types::{Body, Bytes, InterpolatedString, Json, MultiLineString},
};

pub fn bytes_parser<'a>() -> impl Parser<'a, &'a str, Bytes, extra::Err<Rich<'a, char>>> + Clone {
    json_value_parser()
        .map(Bytes::JsonValue)
        // .or(xml_parser()) //TODO when hurl implements syntax for xml bytes
        .or(multiline_string_parser().map(Bytes::MultilineString))
        .or(one_line_string_parser().map(Bytes::OneLineString))
        .or(one_line_base64_parser().map(Bytes::OneLineBase64))
        .or(one_line_file_parser().map(Bytes::OneLineFile))
        .or(one_line_hex_parser().map(Bytes::OneLineHex))
        .labelled("bytes")
        .boxed()
}

fn json_value_parser<'a>() -> impl Parser<'a, &'a str, Json, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser().map(|_| Json::Invalid).boxed()
}

fn multiline_string_parser<'a>(
) -> impl Parser<'a, &'a str, MultiLineString, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser()
        .map(|_| MultiLineString::Json(InterpolatedString { parts: vec![] }))
        .boxed()
}

fn one_line_string_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser()
        .map(|_| InterpolatedString { parts: vec![] })
        .boxed()
}

fn one_line_base64_parser<'a>(
) -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser().map(|_| "TODO".to_string()).boxed()
}

fn one_line_file_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    todo_parser()
        .map(|_| InterpolatedString { parts: vec![] })
        .boxed()
}

fn one_line_hex_parser<'a>() -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone
{
    todo_parser().map(|_| "TODO".to_string()).boxed()
}

pub fn body_parser<'a>() -> impl Parser<'a, &'a str, Body, extra::Err<Rich<'a, char>>> + Clone {
    bytes_parser()
        .map(|bytes| Body { bytes })
        .labelled("body")
        .boxed()
}

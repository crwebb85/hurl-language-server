use chumsky::prelude::*;

use super::{
    primitives::todo_parser,
    types::{Body, Bytes, InterpolatedString, Json, MultiLineString},
};

pub fn bytes_parser() -> impl Parser<char, Bytes, Error = Simple<char>> + Clone {
    json_value_parser()
        .map(Bytes::JsonValue)
        // .or(xml_parser()) //TODO when hurl implements syntax for xml bytes
        .or(multiline_string_parser().map(Bytes::MultilineString))
        .or(one_line_string_parser().map(Bytes::OneLineString))
        .or(one_line_base64_parser().map(Bytes::OneLineBase64))
        .or(one_line_file_parser().map(Bytes::OneLineFile))
        .or(one_line_hex_parser().map(Bytes::OneLineHex))
        .labelled("bytes")
}

fn json_value_parser() -> impl Parser<char, Json, Error = Simple<char>> + Clone {
    todo_parser().map(|_| Json::Invalid)
}

fn multiline_string_parser() -> impl Parser<char, MultiLineString, Error = Simple<char>> + Clone {
    todo_parser().map(|_| MultiLineString::Json(InterpolatedString { parts: vec![] }))
}

fn one_line_string_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    todo_parser().map(|_| InterpolatedString { parts: vec![] })
}

fn one_line_base64_parser() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    todo_parser().map(|_| "TODO".to_string())
}

fn one_line_file_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    todo_parser().map(|_| InterpolatedString { parts: vec![] })
}

fn one_line_hex_parser() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    todo_parser().map(|_| "TODO".to_string())
}

pub fn body_parser() -> impl Parser<char, Body, Error = Simple<char>> + Clone {
    bytes_parser().map(|bytes| Body { bytes }).labelled("body")
}

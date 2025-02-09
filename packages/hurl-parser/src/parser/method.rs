use chumsky::prelude::*;

use super::{
    key_value::value_parser,
    primitives::{ascii_alphabetic_uppercase_parser, lt_parser, sp_parser},
    types::{InterpolatedString, Method},
};

fn method_parser<'a>(
    strict: bool,
) -> impl Parser<'a, &'a str, Method, extra::Err<Rich<'a, char>>> + Clone {
    if strict {
        ascii_alphabetic_uppercase_parser()
            .repeated()
            .at_least(1)
            .collect::<String>()
            .map(|method| Method { value: method })
            .boxed()
    } else {
        text::ident()
            .to_slice()
            .validate(|ident: &str, e, emitter| {
                match ident.find(|c| !char::is_ascii_uppercase(&c)) {
                    Some(index) => emitter.emit(Rich::custom(
                        e.span(),
                        format!(
                            "Invalid character '{}'. Method must be ascii uppercase.",
                            ident.chars().nth(index).unwrap() // We know the character is at the index
                        ),
                    )),
                    None => (),
                };

                Method {
                    value: ident.to_string(),
                }
            })
            .boxed()
    }
}

pub fn method_line_parser<'a>(
    strict: bool,
) -> impl Parser<'a, &'a str, (Method, InterpolatedString), extra::Err<Rich<'a, char>>> + Clone {
    let method_line = method_parser(strict)
        .padded_by(sp_parser().repeated()) //TODO sp is required
        .then(value_parser())
        .then_ignore(lt_parser());
    method_line.boxed()
}

//TODO add tests

//TODO error handle if missing URL

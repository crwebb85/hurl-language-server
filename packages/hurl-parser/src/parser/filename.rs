use crate::parser::template::template_parser;
use crate::parser::types::{InterpolatedString, InterpolatedStringPart};
use chumsky::prelude::*;

pub fn filename_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    let template = template_parser();

    let filename_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
                .or(just('\\').to('\\'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t'))
                .or(just('#').to('#'))
                .or(just(';').to(';'))
                .or(just(' ').to(' '))
                .or(just('{').to('{'))
                .or(just('}').to('}'))
                .or(just(':').to(':'))
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
        .labelled("filename_string_escaped_char");

    let filename_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        // ~[#;{} \n\\]+
        c != &'#' && c != &';' && c != &'{' && c != &'}' && c != &' ' && c != &'\n' && c != &'\\'
    })
    .or(filename_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("filename_str");

    let filename_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("filename_template");

    let filename = filename_str_part
        .or(filename_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k })
        .labelled("filename");

    filename
}

use crate::parser::template::template_parser;
use crate::parser::types::{InterpolatedString, InterpolatedStringPart};
use chumsky::prelude::*;

pub fn quoted_string_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone
{
    let quoted_string_escaped_char = just('\\')
        .ignore_then(
            just('\\')
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
                            char::from_u32(u32::from_str_radix(&digits, 16).unwrap())
                                .unwrap_or_else(|| {
                                    emit(Simple::custom(span, "invalid unicode character"));
                                    '\u{FFFD}' // unicode replacement character
                                })
                        }),
                )),
        )
        .labelled("quoted_string_escaped_char");

    let quoted_str_part = filter::<_, _, Simple<char>>(|c: &char| c != &'"' && c != &'\\')
        .or(quoted_string_escaped_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str);

    let template = template_parser();

    let quoted_template_part = template.map(|t| InterpolatedStringPart::Template(t));

    let quoted_part = quoted_template_part.or(quoted_str_part);

    let quoted_string = just("\"")
        .ignored()
        .then(quoted_part.repeated().at_least(1))
        .then_ignore(just("\""))
        .map(|(_, v)| InterpolatedString { parts: v })
        .labelled("quoted_string");

    quoted_string
}

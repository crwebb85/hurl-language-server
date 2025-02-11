use super::primitives::todo_parser;
use super::types::ResponseSection;
use chumsky::prelude::*;

pub fn response_sections_parser<'a>(
) -> impl Parser<'a, &'a str, Vec<ResponseSection>, extra::Err<Rich<'a, char>>> + Clone {
    // response_section_parser()
    //     .repeated()
    //     .collect::<Vec<ResponseSection>>(),
    todo_parser().or_not().map(|_| vec![]).boxed()

    //TODO sections can only be defined once per entry's request section. So you can't have [Asserts] defined
    //twice and so should be a diagnostic error.
}

//TODO tests

use chumsky::prelude::*;

pub fn sp_parser() -> impl Parser<char, char, Error = Simple<char>> + Clone {
    filter(|c: &char| c.is_whitespace() && (c == &'\t' || c == &' '))
}

pub fn lt_parser() -> impl Parser<char, Option<()>, Error = Simple<char>> + Clone {
    let sp = sp_parser();

    let comment = just('#').then(
        filter::<_, _, Simple<char>>(|c| c != &'\n')
            .repeated()
            .at_least(1),
    );

    sp.repeated()
        .ignored()
        .then_ignore(comment)
        .or_not() // or_not makes the comment optional
        .then_ignore(text::newline().or(end()))
}

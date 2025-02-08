use chumsky::prelude::*;

use super::primitives::{alphanumeric_parser, sp_parser};

pub fn oneline_base64_parser<'a>(
) -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    just("base64,")
        .padded_by(sp_parser().repeated())
        .ignore_then(
            choice((
                alphanumeric_parser(),
                //TODO off-spec \t is not in the spec but is in the official parser and I verified
                //it works
                //TODO \n is in the spec and in the official parser but when I try to use a hurl
                //file with it it allways errors saying "expecting ';'"
                one_of("+-=\n \t"),
            ))
            .repeated()
            .collect::<String>(),
        )
        .then_ignore(just(";"))
}

#[cfg(test)]
mod oneline_base64_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_base64_string() {
        let test_str = r#"base64, VGhpcyBpcyBhIHRlc3Q=;"#;
        assert_debug_snapshot!(
        oneline_base64_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "VGhpcyBpcyBhIHRlc3Q=",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_base64_string_with_extra_spacing() {
        let test_str = r#"base64,  VGhpc  yBpcyBhIHRl  c3Q =  ;"#;
        //TODO I'm not sure if it would be better to remove the whitespace
        //from the parsed base 64 value or leave it alone. Both are valid.
        assert_debug_snapshot!(
        oneline_base64_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "VGhpc  yBpcyBhIHRl  c3Q =  ",
            ),
            errs: [],
        }
        "#,
        );
    }
}

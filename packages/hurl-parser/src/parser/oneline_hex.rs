use super::primitives::sp_parser;
use chumsky::prelude::*;

pub fn oneline_hex_parser<'a>(
) -> impl Parser<'a, &'a str, String, extra::Err<Rich<'a, char>>> + Clone {
    just("hex,")
        .ignore_then(
            text::digits(16)
                .to_slice()
                .padded_by(sp_parser().repeated())
                .map(|s: &str| s.to_string()),
        )
        .then_ignore(just(";"))
}

#[cfg(test)]
mod oneline_hex_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_hex_string() {
        let test_str = r#"hex, 2AFA;"#;
        assert_debug_snapshot!(
        oneline_hex_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "2AFA",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_hex_string_with_extra_spaces() {
        let test_str = r#"hex,   2AFA  ;"#;
        assert_debug_snapshot!(
        oneline_hex_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "2AFA",
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_hex_lowercase_string() {
        let test_str = r#"hex, 2afa;"#;
        assert_debug_snapshot!(
        oneline_hex_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                "2afa",
            ),
            errs: [],
        }
        "#,
        );
    }
}

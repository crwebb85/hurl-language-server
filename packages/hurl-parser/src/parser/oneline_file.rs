use super::filename::filename_parser;
use super::primitives::sp_parser;
use super::types::InterpolatedString;
use chumsky::prelude::*;

//TODO official spec should reuse the "file, filename ;" in file-value since
//it is also the syntax as oneline-filename
pub fn oneline_file_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    just("file,")
        .padded_by(sp_parser().repeated())
        .ignore_then(filename_parser())
        .padded_by(sp_parser().repeated())
        .then_ignore(just(';'))
}

#[cfg(test)]
mod oneline_file_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_oneline_file() {
        let test_str = "file,example.txt;";
        assert_debug_snapshot!(
        oneline_file_parser().then_ignore(end()).parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "example.txt",
                        ),
                    ],
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_oneline_file_with_spaces() {
        let test_str = "file,  example.txt  ;";
        assert_debug_snapshot!(
        oneline_file_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "example.txt",
                        ),
                    ],
                },
            ),
            errs: [],
        }
        "#,
        );
    }
}

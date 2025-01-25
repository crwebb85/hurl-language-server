use super::primitives::escaped_unicode_parser;
use crate::parser::template::template_parser;
use crate::parser::types::{InterpolatedString, InterpolatedStringPart};
use chumsky::prelude::*;

/// Parses a file path
///
/// Note: file paths cannot be files outside the root directory. Where
/// the root directory is by default files directory the hurl file is in. This can
/// be changed withe the hurl --file-root option.
/// TODO: Add LSP Diagnostics for is the file does not exist or if the file is
/// outside the root directory
///
///
/// # Returns
/// The filename parser
///
/// ```
pub fn filename_parser<'a>(
) -> impl Parser<'a, &'a str, InterpolatedString, extra::Err<Rich<'a, char>>> + Clone {
    let filename_escape_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('#').to('#'),
            just(';').to(';'),
            just(' ').to(' '),
            just('{').to('{'),
            just('}').to('}'),
            just(':').to(':'),
        )))
        .or(escaped_unicode_parser())
        .labelled("filename-escaped-char");

    let filename_content = choice((none_of("#;{} \n\\"), filename_escape_char))
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("filename-content");

    let filename = choice((
        filename_content,
        template_parser().map(|t| InterpolatedStringPart::Template(t)),
    ))
    .repeated()
    .at_least(1)
    .collect::<Vec<InterpolatedStringPart>>()
    .map(|k| InterpolatedString { parts: k })
    .labelled("filename");

    filename
}

#[cfg(test)]
mod expr_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_simple_filename() {
        let test_str = "example.txt";
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
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
    fn it_parses_filename_template() {
        let test_str = "{{input_file}}";
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "input_file",
                                    ),
                                    filters: [],
                                },
                            },
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
    fn it_parses_filename_template_in_file_name() {
        let test_str = "./tests/{{testFile1}}";
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "./tests/",
                        ),
                        Template(
                            Template {
                                expr: Expr {
                                    variable: VariableName(
                                        "testFile1",
                                    ),
                                    filters: [],
                                },
                            },
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
    fn it_parses_relative_directory() {
        let test_str = r#"./temp/example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "./temp/example.txt",
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
    fn it_parses_relative_directory_with_parent_dir_operator() {
        let test_str = r#"./temp/../example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "./temp/../example.txt",
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
    fn it_parses_relative_directory_with_multiple_parent_dir_operator() {
        let test_str = r#"./temp/../../example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "./temp/../../example.txt",
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
    fn it_parses_relative_directory_with_leading_parent_dir_operator() {
        let test_str = r#"../example/example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "../example/example.txt",
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
    fn it_parses_absolute_directory() {
        let test_str =
            r#"C:/Users/myuser/Documents/projects/hurl-language-server/examples/example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "C:/Users/myuser/Documents/projects/hurl-language-server/examples/example.txt",
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
    fn it_parses_absolute_directory_with_backslashes() {
        let test_str = r#"C:\\Users\\myuser\\Documents\\projects\\hurl-language-server\\examples\\example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                InterpolatedString {
                    parts: [
                        Str(
                            "C:\\Users\\myuser\\Documents\\projects\\hurl-language-server\\examples\\example.txt",
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
    fn it_errors_invalid_characters() {
        let test_str = r#"#"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''#'' at 0..1 expected filename,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_invalid_character_in_filename() {
        let test_str = r#"test#.txt"#;
        //using ".then(end())" to parse to end of string to test the error message.
        //Otherwise, it would successfully parse the valid part of test and ignore
        //the rest.
        assert_debug_snapshot!(
        filename_parser().then(end()).parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''#'' at 4..5 expected something else, filename-escaped-char, filename-content, template, or end of input,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_invalid_characters_in_filename() {
        let test_strings = vec![r#"#"#, r#";"#, r#"{"#, r#"}"#, r#" "#, "\n", r#"\"#];
        for test_str in test_strings {
            assert!(
                filename_parser().parse(test_str).into_errors().len() > 0,
                r#"The filename parser unexpectedly did not error for the string "{}""#,
                test_str
            );
        }
    }
}

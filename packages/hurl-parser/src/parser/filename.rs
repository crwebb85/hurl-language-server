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
pub fn filename_parser() -> impl Parser<char, InterpolatedString, Error = Simple<char>> + Clone {
    let template = template_parser();

    let filename_escape_char = just('\\')
        .ignore_then(
            just('\\')
                .to('\\')
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
        .labelled("filename-escaped-char");

    let filename_text = filter::<_, _, Simple<char>>(|c: &char| {
        // ~[#;{} \n\\]+
        c != &'#' && c != &';' && c != &'{' && c != &'}' && c != &' ' && c != &'\n' && c != &'\\'
    })
    .labelled("filename-text");

    let filename_content = filename_text
        .or(filename_escape_char)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(InterpolatedStringPart::Str)
        .labelled("filename-content");

    let filename_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t))
        .labelled("filename-template");

    let filename = filename_content
        .or(filename_template_part)
        .repeated()
        .at_least(1)
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
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "example.txt",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_filename_template() {
        let test_str = "{{input_file}}";
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
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
        )
        "#,
        );
    }

    #[test]
    fn it_parses_filename_template_in_file_name() {
        let test_str = "./tests/{{testFile1}}";
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
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
        )
        "#,
        );
    }

    #[test]
    fn it_parses_relative_directory() {
        let test_str = r#"./temp/example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "./temp/example.txt",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_relative_directory_with_parent_dir_operator() {
        let test_str = r#"./temp/../example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "./temp/../example.txt",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_relative_directory_with_multiple_parent_dir_operator() {
        let test_str = r#"./temp/../../example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "./temp/../../example.txt",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_relative_directory_with_leading_parent_dir_operator() {
        let test_str = r#"../example/example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "../example/example.txt",
                    ),
                ],
            },
        )
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
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "C:/Users/myuser/Documents/projects/hurl-language-server/examples/example.txt",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_absolute_directory_with_backslashes() {
        let test_str = r#"C:\\Users\\myuser\\Documents\\projects\\hurl-language-server\\examples\\example.txt"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Ok(
            InterpolatedString {
                parts: [
                    Str(
                        "C:\\Users\\myuser\\Documents\\projects\\hurl-language-server\\examples\\example.txt",
                    ),
                ],
            },
        )
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_characters() {
        let test_str = r#"#"#;
        assert_debug_snapshot!(
        filename_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 0..1,
                    reason: Unexpected,
                    expected: {
                        Some(
                            '\\',
                        ),
                        Some(
                            '{',
                        ),
                    },
                    found: Some(
                        '#',
                    ),
                    label: Some(
                        "filename",
                    ),
                },
            ],
        )
        "#,
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
            @r#"
        Err(
            [
                Simple {
                    span: 4..5,
                    reason: Unexpected,
                    expected: {
                        None,
                        Some(
                            '\\',
                        ),
                        Some(
                            '{',
                        ),
                    },
                    found: Some(
                        '#',
                    ),
                    label: Some(
                        "filename",
                    ),
                },
            ],
        )
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_characters_in_filename() {
        let test_strings = vec![r#"#"#, r#";"#, r#"{"#, r#"}"#, r#" "#, "\n", r#"\"#];
        for test_str in test_strings {
            assert!(
                filename_parser().parse(test_str).is_err(),
                r#"The filename parser unexpectedly did not error for the string "{}""#,
                test_str
            );
        }
    }
}

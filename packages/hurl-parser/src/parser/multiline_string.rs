use chumsky::prelude::*;
use text::ascii::ident;

use super::{
    primitives::{escaped_unicode_parser, sp_parser},
    template::template_parser,
    types::{
        InterpolatedString, InterpolatedStringPart, MultilineString, MultilineStringAttribute,
        MultilineStringType,
    },
};
type Span = SimpleSpan;
type Spanned<T> = (T, Span);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeOrAttributeToken<'src> {
    //MultilineStringTypes
    Base64,
    Hex,
    Json,
    Xml,
    Graphql,
    //MultilineStringAttributes
    Escape,
    NoVariable,
    //Other
    Ident(&'src str),
}

impl std::fmt::Display for TypeOrAttributeToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TypeOrAttributeToken::Base64 => write!(f, "base64"),
            TypeOrAttributeToken::Hex => write!(f, "hex"),
            TypeOrAttributeToken::Json => write!(f, "json"),
            TypeOrAttributeToken::Xml => write!(f, "xml"),
            TypeOrAttributeToken::Graphql => write!(f, "graphql"),
            TypeOrAttributeToken::Escape => write!(f, "escape"),
            TypeOrAttributeToken::NoVariable => write!(f, "novariable"),
            TypeOrAttributeToken::Ident(x) => write!(f, "{}", x),
        }
    }
}

fn multiline_string_header_tokenizor<'src>() -> impl Parser<
    'src,
    &'src str,
    Vec<Spanned<TypeOrAttributeToken<'src>>>,
    extra::Err<Rich<'src, char, Span>>,
> {
    let multiline_header_token = choice((
        just("base64").to(TypeOrAttributeToken::Base64),
        just("hex").to(TypeOrAttributeToken::Hex),
        just("json").to(TypeOrAttributeToken::Json),
        just("xml").to(TypeOrAttributeToken::Xml),
        just("graphql").to(TypeOrAttributeToken::Graphql),
        just("escape").to(TypeOrAttributeToken::Escape),
        just("novariable").to(TypeOrAttributeToken::NoVariable),
        ident().map(|s| TypeOrAttributeToken::Ident(s)),
    ))
    .map_with(|tok, e| (tok, e.span()))
    .boxed();

    multiline_header_token
        .separated_by(just(",").padded_by(sp_parser().repeated()))
        .allow_trailing() //TODO trailing comma is off-spec but is allowed
        .collect::<Vec<_>>()
}

fn multiline_string_escaped_char_parser<'a>(
) -> impl Parser<'a, &'a str, char, extra::Err<Rich<'a, char>>> + Clone {
    let multiline_string_escaped_char = just('\\')
        .ignore_then(choice((
            just('\\').to('\\'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('`').to('`'),
        )))
        .or(escaped_unicode_parser())
        .labelled("multiline-string-escaped-char");
    multiline_string_escaped_char.boxed()
}

fn multiline_string_header_parser<'a>() -> impl Parser<
    'a,
    &'a str,
    (Option<MultilineStringType>, Vec<MultilineStringAttribute>),
    extra::Err<Rich<'a, char>>,
> + Clone {
    let multiline_string_header =
        multiline_string_header_tokenizor().validate(|tokens: Vec<Spanned<TypeOrAttributeToken>>, _, emitter| {
            match tokens.first() {
                Some((token, span)) => {
                    let multiline_string_type = match token {
                        TypeOrAttributeToken::Base64 => Some(MultilineStringType::Base64),
                        TypeOrAttributeToken::Hex => Some(MultilineStringType::Hex),
                        TypeOrAttributeToken::Json => Some(MultilineStringType::Json),
                        TypeOrAttributeToken::Xml => Some(MultilineStringType::Xml),
                        TypeOrAttributeToken::Graphql => Some(MultilineStringType::Graphql),
                        TypeOrAttributeToken::Escape => None,
                        TypeOrAttributeToken::NoVariable => None,
                        TypeOrAttributeToken::Ident(s) => {
                                emitter.emit(Rich::custom(
                                    *span,
                                    format!(
                                        "Unknown multiline string type or attribute `{}`.",
                                        token.to_string()
                                    ),
                                ));
                            Some(MultilineStringType::Unknown(s.to_string()))
                        },
                    };
                    let mut token_iterator = tokens.into_iter();
                    if multiline_string_type.is_some() {
                        //Skip the first token since it was a
                        //MultilineStringType token and not a MultilineStringAttribute
                        token_iterator.next();
                    }
                    let attributes = token_iterator
                        .map(|(token, span)| match token {
                            TypeOrAttributeToken::Base64
                            | TypeOrAttributeToken::Hex
                            | TypeOrAttributeToken::Json
                            | TypeOrAttributeToken::Xml
                            | TypeOrAttributeToken::Graphql => {
                                emitter.emit(Rich::custom(
                                    span,
                                    format!(
                                        "Found multiline string type `{}` in the multiline string attribute section.",
                                        token.to_string()
                                    ),
                                ));
                                MultilineStringAttribute::Unknown(token.to_string())
                            },

                            TypeOrAttributeToken::Ident(_) => {
                                emitter.emit(Rich::custom(
                                    span,
                                    format!(
                                        "Found unknown multiline string attribute `{}`.",
                                        token.to_string()
                                    )
                                ));
                                MultilineStringAttribute::Unknown(token.to_string())
                            }
                            TypeOrAttributeToken::Escape => MultilineStringAttribute::Escape,
                            TypeOrAttributeToken::NoVariable => MultilineStringAttribute::NoVariable,
                        })
                        .collect();
                    (multiline_string_type, attributes)
                }
                None => (None, vec![]),
            }
        }).boxed();

    multiline_string_header.then_ignore(text::newline()).boxed() //TODO off-spec official grammer allows
                                                                 //full lt_parser() but hurl only allows
                                                                 //a single newline of \n or \r\n
}

pub fn multiline_string_parser<'a>(
) -> impl Parser<'a, &'a str, MultilineString, extra::Err<Rich<'a, char>>> + Clone {
    let multiline_string_content = choice((
        none_of("\\{`"),
        multiline_string_escaped_char_parser(),
        //opening curly brackes are valid as long as they are not followed by a
        //second curly bracket since two denote the start of a template
        //(this observation isn't explicit in the grammer but was determined
        just('{').then(just('{').not().rewind()).to('{'),
        //backtick quotes are valid as long as they are not followed by a
        //two more backticks since a total of three denote the end of the multiline
        //string
        just('`').then(just("``").not().rewind()).to('`'),
    ))
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str)
    .labelled("multiline-string-content");

    let multiline_string = multiline_string_header_parser()
        .then(
            choice((
                multiline_string_content,
                template_parser().map(|t| InterpolatedStringPart::Template(t)),
            ))
            .repeated()
            .collect::<Vec<InterpolatedStringPart>>()
            .map(|v| InterpolatedString { parts: v }),
        )
        //TODO off-spec there is supposed to be an lt_parser() before the closing
        //multistring quote sequence of ```. The official parser however does not require it
        // .then_ignore(lt_parser())
        .boxed();

    multiline_string
        .delimited_by(just("```"), just("```"))
        .map(|((string_type, attributes), content)| MultilineString {
            r#type: string_type,
            attributes,
            content,
        })
        .boxed()
}

#[cfg(test)]
mod multiline_string_header_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_empty_multiline_string_header() {
        let test_str = "\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    None,
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_base64() {
        let test_str = "base64\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Base64,
                    ),
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_hex() {
        let test_str = "hex\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Hex,
                    ),
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_json() {
        let test_str = "json\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Json,
                    ),
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_xml() {
        let test_str = "xml\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Xml,
                    ),
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_graphql() {
        let test_str = "graphql\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Graphql,
                    ),
                    [],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_escape() {
        let test_str = "escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    None,
                    [
                        Escape,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_novariable() {
        let test_str = "novariable\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    None,
                    [
                        NoVariable,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_recovers_multiline_string_header_unknown_type_or_attribute() {
        let test_str = "unknown_attribute\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    Some(
                        Unknown(
                            "unknown_attribute",
                        ),
                    ),
                    [],
                ),
            ),
            errs: [
                Unknown multiline string type or attribute `unknown_attribute`. at 0..17,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_multiline_string_header_with_type_and_multiple_attributes() {
        let test_str = "json,novariable,escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Json,
                    ),
                    [
                        NoVariable,
                        Escape,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_with_type_and_multiple_attributes_with_extra_space() {
        let test_str = "json  ,   novariable   ,   escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Json,
                    ),
                    [
                        NoVariable,
                        Escape,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_errors_multiline_string_header_with_comment() {
        //TODO This is off-spec since the official grammer does allow comments but hurl does not
        let test_str = "json,novariable,escape #this is a comment\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''#'' at 23..24 expected spacing, or '','',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_with_type_and_multiple_attributes_with_extra_space_and_trailing_comma(
    ) {
        //TODO this is off-spec but is allowed by hurl
        let test_str = "json  ,   novariable   ,   escape ,  \n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    Some(
                        Json,
                    ),
                    [
                        NoVariable,
                        Escape,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_with_no_type_and_multiple_attributes() {
        let test_str = "novariable,escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    None,
                    [
                        NoVariable,
                        Escape,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_parses_multiline_string_header_with_no_type_and_multiple_attributes_with_extra_space() {
        let test_str = "novariable  ,  escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: Some(
                (
                    None,
                    [
                        NoVariable,
                        Escape,
                    ],
                ),
            ),
            errs: [],
        }
        ",
        );
    }

    #[test]
    fn it_recovers_multiline_string_header_unknown_type_and_attributes() {
        let test_str = "unknown_type, novariable, unknown, escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                (
                    Some(
                        Unknown(
                            "unknown_type",
                        ),
                    ),
                    [
                        NoVariable,
                        Unknown(
                            "unknown",
                        ),
                        Escape,
                    ],
                ),
            ),
            errs: [
                Unknown multiline string type or attribute `unknown_type`. at 0..12,
                Found unknown multiline string attribute `unknown`. at 26..33,
            ],
        }
        "#,
        );
    }

    #[test]
    fn it_recovers_multiline_string_header_with_invalid_text() {
        let test_str = "json,novariable, *ignored,escape\n";
        assert_debug_snapshot!(
        multiline_string_header_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''*'' at 17..18 expected identifier, or newline,
            ],
        }
        ",
        );
    }
}

#[cfg(test)]
mod multiline_string_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_errors_multilinestring_without_line_terminator_in_header() {
        let test_str = r#"``````"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''`'' at 3..4 expected identifier, or newline,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_empty_multilinestring() {
        let test_str = r#"```

```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: None,
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "\n",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_multiple_lines() {
        let test_str = r#"```
    this is some text
    another line
    hello world
                ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: None,
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "    this is some text\n    another line\n    hello world\n                ",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_does_not_parse_comments_in_multiline_string() {
        let test_str = r#"```json
5 # test
# test
# testing ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: Some(
                        Json,
                    ),
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "5 # test\n# test\n# testing ",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_multiline_string_with_backtics() {
        let test_str = r#"```
this `text` has some funny ``quotes``
            ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: None,
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "this `text` has some funny ``quotes``\n            ",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_multiline_string_with_template() {
        let test_str = r#"```json
    {
        {{key}}: {{value}}
    }
            ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: Some(
                        Json,
                    ),
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "    {\n        ",
                            ),
                            Template(
                                Template {
                                    expr: Expr {
                                        variable: VariableName(
                                            "key",
                                        ),
                                        filters: [],
                                    },
                                },
                            ),
                            Str(
                                ": ",
                            ),
                            Template(
                                Template {
                                    expr: Expr {
                                        variable: VariableName(
                                            "value",
                                        ),
                                        filters: [],
                                    },
                                },
                            ),
                            Str(
                                "\n    }\n            ",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_unclosed_template_in_multiline_string() {
        let test_str = r#"```json
"test{{"
```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''{'' at 14..15 expected spacing, expr, or something else,
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_unclosed_template_in_multiline_string_with_attempted_escaping() {
        let test_str = r#"```json
"test\{{"
```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''{'' at 14..15 expected ''\\'', ''b'', ''f'', ''n'', ''r'', ''t'', ''`'', or ''u'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_parses_escape_sequences_in_multiline_string() {
        let test_str = r#"```
            escapedchars(\`, \\, \b, \f, \r\n, \t)
            ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: None,
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "            escapedchars(`, \\, \u{8}, \u{c}, \r\n, \t)\n            ",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_escaped_emoji_in_multiline_string() {
        let test_str = r#"```
            escapedemoji(\u{0001}\u{F600})
            ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                MultilineString {
                    type: None,
                    attributes: [],
                    content: InterpolatedString {
                        parts: [
                            Str(
                                "            escapedemoji(\u{1}\u{f600})\n            ",
                            ),
                        ],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_invalid_escaped_unicode_in_multiline_string() {
        //unicode must include 4 hex digits to be valid. 'H' is not a valid hex digit.
        let test_str = r#"```
            escapedemoji(\u{FFFH})
    ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''H'' at 35..36 expected digit, or ''}'',
            ],
        }
        ",
        );
    }

    #[test]
    fn it_errors_invalid_escape_char_in_multiline_string() {
        // g is not a valid character for escaping
        let test_str = r#"```
            invalidescapechar:\g
            ```"#;
        assert_debug_snapshot!(
        multiline_string_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''g'' at 35..36 expected ''\\'', ''b'', ''f'', ''n'', ''r'', ''t'', ''`'', or ''u'',
            ],
        }
        ",
        );
    }
}

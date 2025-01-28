use super::{
    expr::expr_parser, primitives::sp_parser, quoted_string::generic_quoted_string_parser,
};
use crate::parser::types::Template;
use chumsky::prelude::*;

pub fn template_parser<'a>(
) -> impl Parser<'a, &'a str, Template, extra::Err<Rich<'a, char>>> + Clone {
    let template = recursive(|template| {
        let quoted_string = generic_quoted_string_parser(template);
        let expr = expr_parser(quoted_string);
        expr.padded_by(sp_parser().repeated())
            .delimited_by(just("{{"), just("}}"))
            .map(|captured_expr| Template {
                expr: captured_expr,
            })
    })
    .labelled("template");

    template.boxed()
}

#[cfg(test)]
mod template_tests {

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_template() {
        let test_str = "{{key}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Template {
                    expr: Expr {
                        variable: VariableName(
                            "key",
                        ),
                        filters: [],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_template_with_extra_whitespace() {
        let test_str = "{{   key  }}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Template {
                    expr: Expr {
                        variable: VariableName(
                            "key",
                        ),
                        filters: [],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_template_with_dashed_variable() {
        let test_str = "{{api-key}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Template {
                    expr: Expr {
                        variable: VariableName(
                            "api-key",
                        ),
                        filters: [],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_parses_template_with_underscore_variable() {
        let test_str = "{{api_key}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        ParseResult {
            output: Some(
                Template {
                    expr: Expr {
                        variable: VariableName(
                            "api_key",
                        ),
                        filters: [],
                    },
                },
            ),
            errs: [],
        }
        "#,
        );
    }

    #[test]
    fn it_errors_template_with_number_variable() {
        let test_str = "{{1}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r"
        ParseResult {
            output: None,
            errs: [
                found ''1'' at 2..3 expected spacing, or expr,
            ],
        }
        ",
        );
    }
}

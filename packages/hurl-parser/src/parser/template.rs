use super::{expr::expr_parser, quoted_string::generic_quoted_string_parser};
use crate::parser::types::Template;
use chumsky::prelude::*;

pub fn template_parser() -> impl Parser<char, Template, Error = Simple<char>> + Clone {
    let template = recursive(|template| {
        let quoted_string = generic_quoted_string_parser(template);
        let expr = expr_parser(quoted_string);
        just("{")
            .ignored()
            .then_ignore(just("{"))
            .then(expr)
            .then_ignore(just("}"))
            .then_ignore(just("}"))
            .map(|(_, captured_expr)| Template {
                expr: captured_expr,
            })
    })
    .labelled("template");

    template
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
        Ok(
            Template {
                expr: Expr {
                    variable: VariableName(
                        "key",
                    ),
                    filters: [],
                },
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_template_with_dashed_variable() {
        let test_str = "{{api-key}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        Ok(
            Template {
                expr: Expr {
                    variable: VariableName(
                        "api-key",
                    ),
                    filters: [],
                },
            },
        )
        "#,
        );
    }

    #[test]
    fn it_parses_template_with_underscore_variable() {
        let test_str = "{{api_key}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        Ok(
            Template {
                expr: Expr {
                    variable: VariableName(
                        "api_key",
                    ),
                    filters: [],
                },
            },
        )
        "#,
        );
    }

    #[test]
    fn it_errors_template_with_number_variable() {
        let test_str = "{{1}}";
        assert_debug_snapshot!(
        template_parser().parse(test_str),
            @r#"
        Err(
            [
                Simple {
                    span: 2..3,
                    reason: Unexpected,
                    expected: {},
                    found: Some(
                        '1',
                    ),
                    label: Some(
                        "variable_name",
                    ),
                },
            ],
        )
        "#,
        );
    }
}

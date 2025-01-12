use chumsky::{error::Simple, prelude::{choice, filter, just}, text, Parser};

use super::{primitives::sp_parser, types::{Expr, ExprValue, FilterFunction, InterpolatedString}};

pub fn variable_name_parser() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    let variable_name = filter::<_, _, Simple<char>>(char::is_ascii_alphabetic)
        .then(
            filter::<_, _, Simple<char>>(|c: &char| {
                c.is_ascii_alphanumeric() || c == &'_' || c == &'-'
            })
            .repeated()
            .collect::<String>(),
        )
        .map(|(c, chars)| format!("{}{}", c, chars));
    variable_name
}

pub fn expr_parser<T: Parser<char, InterpolatedString, Error = Simple<char>> + Clone>(
    quoted_string: T,
    ) -> impl Parser<char, Expr, Error = Simple<char>> + Clone {

    let sp = sp_parser();
    let variable_name = variable_name_parser();
    let expr_function = choice::<_, Simple<char>>([
        text::keyword("getEnv").to(ExprValue::FunctionName("getEnv".to_owned())), 
        text::keyword("newDate").to(ExprValue::FunctionName("newDate".to_owned())),
        text::keyword("newUuid").to(ExprValue::FunctionName("newUuid".to_owned()))
    ]);

    let expr_variable = expr_function.or(variable_name.map(ExprValue::VariableName));


    let decode_filter_function = just("decode")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Decode { encoding: s});

    let format_filter_function = just("format")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Format { fmt: s});

    let jsonpath_filter_function = just("jsonpath")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::JsonPath { expr: s });

    let nth_filter_function = just("nth")
        .then_ignore(sp.clone())
        .then(text::int(10))
        .map(|(_, n)| FilterFunction::Nth { 
            nth: n.parse::<u64>()
                .expect("TODO implement error recovery for invalid integers used in the Nth filter function argument") 
        });

    let regex_filter_function = just("regex")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Regex { value: s });

    let split_filter_function = just("split")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::Split { sep: s });
        
    let replace_filter_function = just("replace")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|((_, old), new)| FilterFunction::Replace { old_value: old, new_value: new });

    let todate_filter_function = just("toDate")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::ToDate { fmt: s });

    let xpath_filter_function = just("xpath")
        .then_ignore(sp.clone())
        .then(quoted_string.clone())
        .map(|(_, s)| FilterFunction::XPath { expr: s });

    let filter_function = choice::<_, Simple<char>>([
        just("count").to(FilterFunction::Count), 
        just("daysAfterNow").to(FilterFunction::DaysAfterNow),
        just("daysBeforeNow").to(FilterFunction::DaysBeforeNow),
        just("htmlEscape").to(FilterFunction::HtmlEscape),
        just("htmlUnescape").to(FilterFunction::HtmlUnescape),
        just("toFloat").to(FilterFunction::ToFloat),
        just("toInt").to(FilterFunction::ToInt),
        just("urlDecode").to(FilterFunction::UrlDecode),
        just("urlEncode").to(FilterFunction::UrlEncode),
    ]).or(decode_filter_function)
    .or(format_filter_function)
    .or(jsonpath_filter_function)
    .or(nth_filter_function)
    .or(regex_filter_function)
    .or(split_filter_function)
    .or(replace_filter_function)
    .or(todate_filter_function)
    .or(xpath_filter_function);

    let expr = expr_variable
    .then_ignore(sp.clone().or_not())
    .then(filter_function.separated_by(sp.clone()))
    .map( |(expr_var, filter_funcs)| Expr {
        variable: expr_var,
        filters: filter_funcs
    });
    expr
}


#[cfg(test)]
mod expr_tests {

    use crate::parser::quoted_string::quoted_string_parser;

    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parses_variable_name_in_expr() {
        let test_str = "key";
        let quoted_string = quoted_string_parser();
        assert_debug_snapshot!(
        expr_parser(quoted_string).parse(test_str),
            @r#"
        Ok(
            Expr {
                variable: VariableName(
                    "key",
                ),
                filters: [],
            },
        )
        "#,
        );
    }

}

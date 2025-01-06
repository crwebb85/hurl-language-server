use chumsky::prelude::*;
use crate::parser::types::{
     Expr, FilterFunction, InterpolatedString, InterpolatedStringPart, Template, ExprValue
};
use super::primitives::sp_parser;

pub fn template_parser() -> impl Parser<char, Template, Error = Simple<char>> + Clone {

    let sp = sp_parser();
    let quoted_string_escaped_char = just('\\').ignore_then(
        just('\\')
            .or(just('\\').to('\\'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t'))
            .or(just('u').ignore_then(
                filter(|c: &char| c.is_digit(16))
                    .repeated()
                    .exactly(4)
                    .collect::<String>()
                    .validate(|digits, span, emit| {
                        char::from_u32(u32::from_str_radix(&digits, 16).unwrap()).unwrap_or_else(
                            || {
                                emit(Simple::custom(span, "invalid unicode character"));
                                '\u{FFFD}' // unicode replacement character
                            },
                        )
                    }),
            )),
    ).labelled("quoted_string_escaped_char");

    let quoted_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c != &'"' && c != &'\\' 
    })
    .or(quoted_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);

    let variable_name = 
        filter::<_, _, Simple<char>>(char::is_ascii_alphanumeric)
            .repeated()
            .at_least(1).collect::<String>();

    let expr_function = choice::<_, Simple<char>>([
        text::keyword("getEnv").to(ExprValue::FunctionName("getEnv".to_owned())), 
        text::keyword("newDate").to(ExprValue::FunctionName("newDate".to_owned())),
        text::keyword("newUuid").to(ExprValue::FunctionName("newUuid".to_owned()))
    ]);

    let expr_variable = expr_function.or(variable_name.map(ExprValue::VariableName));

    let template = recursive(|template| {

        let quoted_template_part = template
            .map(|t| InterpolatedStringPart::Template(t));

        let quoted_part = quoted_template_part
            .or(quoted_str_part);

        let quoted_string = just("\"")
            .ignored()
            .then(quoted_part.repeated().at_least(1))
            .then_ignore(just("\""))
            .map(|(_, v)| InterpolatedString { parts: v }).labelled("quoted_string");

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

        just("{")
        .ignored()
        .then_ignore(just("{"))
        .then(expr)
        .then_ignore(just("}"))
        .then_ignore(just("}"))
        .map(|(_, captured_expr)| Template {
            expr: captured_expr,
        })
    }).labelled("template");

    template
}


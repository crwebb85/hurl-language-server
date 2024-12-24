pub mod types;
use chumsky::prelude::*;
use text::TextParser;
use types::{
    Entry, Expr, FilterFunction, InterpolatedString, InterpolatedStringPart, KeyValue, Method, Request, Template, ValueString, Variable
};
#[cfg(test)]
mod test;

pub fn ast_parser() -> impl Parser<char, Vec<Entry>, Error = Simple<char>> {
    let method = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|value| Method { value })
        .padded();

    let sp = filter(|c: &char| c.is_whitespace() && (c == &'\t' || c == &' '));

    let comment = just('#').then(
        filter::<_, _, Simple<char>>(|c| c != &'\n')
            .repeated()
            .at_least(1),
    );

    let lt = sp
        .clone()
        .repeated()
        .then(comment)
        .or_not() // or_not makes the comment optional
        .then(text::newline().or(end()));

    let url = take_until(lt.clone())
        .map(|(url_chars, _)| url_chars)
        .collect::<String>()
        .map(|url_string| ValueString { value: url_string });

    let function = choice::<_, Simple<char>>([
        text::keyword("getEnv").to(Variable::FunctionName("getEnv".to_owned())), 
        text::keyword("newDate").to(Variable::FunctionName("newDate".to_owned())),
        text::keyword("newUuid").to(Variable::FunctionName("newUuid".to_owned()))
    ]);

    let variable_name = 
        filter::<_, _, Simple<char>>(char::is_ascii_alphanumeric)
            .repeated()
            .at_least(1).collect::<String>().map(Variable::VariableName);

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
    );

    let quoted_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c != &'"' && c != &'\\' 
    })
    .or(quoted_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);


    // TODO handle recursion for templates
    // let quoted_template_part = template
    //     .clone()
    //     .map(|t| InterpolatedStringPart::Template(t));
    //
    // let quoted_part = quoted_template_part
    //     .or(quoted_str_part)
    //     .or(quoted_brackets);
    // let quoted_part = quoted_str_part;

    let quoted_string = quoted_str_part
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v });


    let decode_filter_function = text::keyword("decode")
        .then_ignore(sp.clone())
        .then_ignore(just("\""))
        .then(quoted_string)
        .then_ignore(just("\""))
        .map(|(_, s)| FilterFunction::Decode { encoding: s});

    let format_filter_function = text::keyword("format")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|(_, s)| FilterFunction::Format { fmt: s});

    let jsonpath_filter_function = text::keyword("jsonpath")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|(_, s)| FilterFunction::JsonPath { expr: s });
    let nth_filter_function = text::keyword("nth")
        .then_ignore(sp.clone())
        .then(text::int(10))
        .map(|(_, n)| FilterFunction::Nth { 
            nth: n.parse::<u64>()
                .expect("TODO implement error recovery for invalid integers used in the Nth filter function argument") 
        });
    let regex_filter_function = text::keyword("regex")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|(_, s)| FilterFunction::Regex { value: s });
    let split_filter_function = text::keyword("split")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|(_, s)| FilterFunction::Split { sep: s });
    let replace_filter_function = text::keyword("replace")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|((_, old), new)| FilterFunction::Replace { old_value: old, new_value: new });
    let todate_filter_function = text::keyword("toDate")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|(_, s)| FilterFunction::ToDate { fmt: s });
    let xpath_filter_function = text::keyword("xpath")
        .then_ignore(sp.clone())
        .then(quoted_string)
        .map(|(_, s)| FilterFunction::XPath { expr: s });

    let filter_function = choice::<_, Simple<char>>([
        text::keyword("count").to(FilterFunction::Count), 
        text::keyword("daysAfterNow").to(FilterFunction::DaysAfterNow),
        text::keyword("daysBeforeNow").to(FilterFunction::DaysBeforeNow),
        text::keyword("htmlEscape").to(FilterFunction::HtmlEscape),
        text::keyword("htmlUnescape").to(FilterFunction::HtmlUnescape),
        text::keyword("toFloat").to(FilterFunction::ToFloat),
        text::keyword("toInt").to(FilterFunction::ToInt),
        text::keyword("urlDecode").to(FilterFunction::UrlDecode),
        text::keyword("urlEncode").to(FilterFunction::UrlEncode),
    ])
        .or(decode_filter_function)
        .or(format_filter_function)
        .or(jsonpath_filter_function)
        .or(nth_filter_function)
        .or(regex_filter_function)
        .or(split_filter_function)
        .or(replace_filter_function)
        .or(todate_filter_function)
        .or(xpath_filter_function);

    let expr_variable = function.or(variable_name);

    let expr = expr_variable
        .then_ignore(sp.clone().or_not())
        .then(filter_function.separated_by(sp.clone()))
        .map( |(expr_var, filter_funcs)| Expr {
            variable: expr_var,
            filters: filter_funcs
        });

    let template = just("{")
        .ignored()
        .then_ignore(just("{"))
        .then(expr)
        .then_ignore(just("}"))
        .then_ignore(just("}"))
        .map(|(_, captured_expr)| Template {
            expr: captured_expr,
        }); 

    let key_string_escaped_char = just('\\').ignore_then(
        //TODO for some reason when I test hurl files with the hurl cli using
        //these escape sequences I get errors. I need to investivate if that is
        //a version issue or if my understanding on this grammar is wrong
        just('\\')
            .or(just('#').to('#'))
            .or(just(':').to(':'))
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
    );

    let key_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric()
            || c == &'_'
            || c == &'-'
            || c == &'.'
            || c == &'['
            || c == &']'
            || c == &'@'
            || c == &'$'
    })
    .or(key_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);

    let key_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t));

    let key = key_str_part
        .or(key_template_part)
        .repeated()
        .at_least(1)
        .map(|k| InterpolatedString { parts: k });

    let value_string_escaped_char = just('\\').ignore_then(
        just('\\')
            .or(just('#').to('#'))
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
    );

    let value_str_part = filter::<_, _, Simple<char>>(|c: &char| {
        c != &'#' && c != &'\n' && c != &'\\' 
            //currly brackets while allowed will be handled after trying to parse
            //as a template
            && c != &'{' && c != &'}'
    })
    .or(value_string_escaped_char)
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);


    let value_brackets = filter::<_, _, Simple<char>>(|c: &char| {
        c == &'{' || c == &'}'
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .map(InterpolatedStringPart::Str);

    let value_template_part = template
        .clone()
        .map(|t| InterpolatedStringPart::Template(t));

    let value_part = value_template_part
        .or(value_str_part)
        .or(value_brackets);

    let value = value_part
        .repeated()
        .at_least(1)
        .map(|v| InterpolatedString { parts: v });

    let key_value = key
        .then_ignore(just(':'))
        .then_ignore(sp.clone().repeated())//TODO: I think this is an offspec sp
        .then(value)
        .map(|(key, value)| KeyValue { key, value });

    let header = key_value.repeated();

    let request = sp.clone().repeated().ignore_then(method.then(url).then(header).map(
        |((method_value, url_value_string), headers)| Request {
            method: method_value,
            url: url_value_string,
            header: headers,
        },
    ));
    let entry = request.map(|request_value| Entry {
        request: Box::new(request_value),
        response: None,
    });
    entry.repeated().then_ignore(lt.clone())
}

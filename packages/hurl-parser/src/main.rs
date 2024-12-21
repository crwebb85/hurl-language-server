use chumsky::prelude::*;
use text::TextParser;

use std::ops::Range;

#[derive(Debug)]
pub struct ImCompleteSemanticToken {
    pub start: usize,
    pub length: usize,
    pub token_type: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Method {
    value: String,
    // TODO add a trait to validate if method is valid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ValueString {
    value: String,
    // TODO add other fields
    // variables: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Header {
    key: String,
    value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Request {
    method: Method,
    url: ValueString,
    header: Vec<Header>,
    //TODO define/parse remaining fields
    // request_sections: Vec<RequestSection>,
    // body: Body,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Response {
    //TODO fill remaining fields
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    request: Box<Request>,
    response: Option<Box<Response>>,
}

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);

pub fn ast_parser() -> impl Parser<char, Vec<Entry>, Error = Simple<char>> {
    let method = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(|value| Method { value })
        .padded();

    let sp = choice::<_, Simple<char>>([text::keyword(" "), text::keyword("\t")]);

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

    let request = method
        .then(url)
        .map(|(method_value, url_value_string)| Request {
            method: method_value,
            url: url_value_string,
            header: vec![],
        });
    let entry = request.map(|request_value| Entry {
        request: Box::new(request_value),
        response: None,
    });
    entry.repeated().then_ignore(lt.clone())
}

fn main() {
    println!("hi");

    //For quick debugging
    let dummy_parser = ast_parser();

    let src = "GET\nhttps://example.org";
    let ast_result = dummy_parser.parse(src);
    println!("{:?}", ast_result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    #[test]
    fn it_parse_simple_get() {
        let test_str = "GET https://example.org";
        assert_debug_snapshot!(
        ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
            // "we are testing that a single line entry without a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_trailing_newline() {
        let test_str = "GET https://example.org\n";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
            // "we are testing that a single line entry with a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_newline_after_method() {
        let test_str = "GET\nhttps://example.org";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
            // "we are testing that a single line entry with a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_newline_after_method_and_after_url() {
        let test_str = "GET\nhttps://example.org\n";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
            // "we are testing that a single line entry with a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_extra_whitespace() {
        let test_str = "GET\n https://example.org";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "GET",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
            // "we are testing that a single line entry with extra whitespace correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_post() {
        let test_str = "POST https://example.org";
        assert_debug_snapshot!(
            ast_parser().parse(test_str),
            @r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "POST",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "#,
            // "we are testing that a single line post entry parses correctly"
        );
    }
    #[test]
    fn it_parses_unknown_method() {
        let test_str = "FOO https://example.org";
        assert_debug_snapshot!(ast_parser().parse(test_str),@r#"
        Ok(
            [
                Entry {
                    request: Request {
                        method: Method {
                            value: "FOO",
                        },
                        url: ValueString {
                            value: "https://example.org",
                        },
                        header: [],
                    },
                    response: None,
                },
            ],
        )
        "# );
    }
}

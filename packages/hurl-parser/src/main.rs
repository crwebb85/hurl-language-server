use chumsky::prelude::*;
use text::TextParser;

use core::fmt;
use std::ops::Range;

#[derive(Debug)]
pub struct ImCompleteSemanticToken {
    pub start: usize,
    pub length: usize,
    pub token_type: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AstNode {
    HurlFile,
    Entry {
        request: Box<AstNode>,
        response: Option<Box<AstNode>>,
    },
    Request {
        method: Box<AstNode>,
        url: Box<AstNode>,
    },
    // Response,
    Method(String),
    // Version,
    // Status,
    // Header,
    // Body,
    // RequestSection,
    //TODO
    // basic-auth-section
    // |query-string-params-section
    // |form-params-section
    // |multipart-form-data-section
    // |cookies-section
    // |options-section
    // ResponseSection,
    //TODO
    //  captures-section
    // |asserts-section
    // KeyValue,
    // KeyString(String),
    ValueString(String),
}

impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //TODO replace with correct logic
        match self {
            AstNode::HurlFile => write!(f, "HurlFile"),
            AstNode::Entry {
                request: _,
                response: _,
            } => write!(f, "entry"),
            AstNode::Request { method: _, url: _ } => write!(f, "request"),
            AstNode::Method(s) => write!(f, "{}", s),
            AstNode::ValueString(s) => write!(f, "{}", s),
        }
    }
}

pub type Span = Range<usize>;
pub type Spanned<T> = (T, Span);

pub fn entry_parser() -> impl Parser<char, AstNode, Error = Simple<char>> {
    let method = filter::<_, _, Simple<char>>(char::is_ascii_uppercase)
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(AstNode::Method)
        .padded();

    let url = take_until(text::newline().or(end()))
        .map(|(url_chars, _)| url_chars)
        .collect::<String>()
        .map(AstNode::ValueString);
    let request_method_line = method.then(url).then_ignore(end());
    let request = request_method_line.map(|(method, url)| AstNode::Request {
        method: Box::new(method),
        url: Box::new(url),
    });
    let entry = request.map(|request| AstNode::Entry {
        request: Box::new(request),
        response: None,
    });
    entry
}

fn main() {
    println!("hi");

    //For quick debugging
    let dummy_parser = entry_parser();

    let src = "GET\nhttps://example.org";
    let ast_result = dummy_parser.parse(src);
    println!("{:?}", ast_result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parse_simple_get() {
        let test_str = "GET https://example.org";
        assert_eq!(
        entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("GET".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that a single line entry without a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_trailing_newline() {
        let test_str = "GET https://example.org\n";
        assert_eq!(
            entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("GET".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that a single line entry with a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_newline_after_method() {
        let test_str = "GET\nhttps://example.org";
        assert_eq!(
            entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("GET".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that a single line entry with a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_newline_after_method_and_after_url() {
        let test_str = "GET\nhttps://example.org\n";
        assert_eq!(
            entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("GET".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that a single line entry with a linefeed at the end correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_get_with_extra_whitespace() {
        let test_str = "GET\n https://example.org";
        assert_eq!(
            entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("GET".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that a single line entry with extra whitespace correctly parses"
        );
    }

    #[test]
    fn it_parses_simple_post() {
        let test_str = "POST https://example.org";
        assert_eq!(
            entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("POST".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that a single line post entry parses correctly"
        );
    }

    #[test]
    fn it_parses_unknown_method() {
        let test_str = "FOO https://example.org";
        assert_eq!(
            entry_parser().parse(test_str),
            Ok(AstNode::Entry {
                request: Box::new(AstNode::Request {
                    method: Box::new(AstNode::Method("FOO".to_owned())),
                    url: Box::new(AstNode::ValueString("https://example.org".to_owned()))
                }),
                response: None
            }),
            "we are testing that an unknow method parses correctly"
        );
    }
}

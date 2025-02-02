use std::collections::HashMap;

pub enum ImCompleteCompletionItem {
    Keyword(String),
    Snippet(String, String),
}

pub enum RequestMethods {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

const GET_SNIPPET: &str = r#"GET ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const HEAD_SNIPPET: &str = r#"HEAD ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const POST_SNIPPET: &str = r#"POST ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const PUT_SNIPPET: &str = r#"PUT ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const DELETE_SNIPPET: &str = r#"DELETE ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const CONNECT_SNIPPET: &str = r#"CONNECT ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const OPTIONS_SNIPPET: &str = r#"OPTIONS ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const TRACE_SNIPPET: &str = r#"TRACE ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;
const PATCH_SNIPPET: &str = r#"PATCH ${1:{{host\}\}}
$2
HTTP ${3:200}
"#;

const METHOD_SNIPPETS: [(&str, &str); 9] = [
    ("GET", GET_SNIPPET),
    ("HEAD", HEAD_SNIPPET),
    ("POST", POST_SNIPPET),
    ("PUT", PUT_SNIPPET),
    ("DELETE", DELETE_SNIPPET),
    ("CONNECT", CONNECT_SNIPPET),
    ("OPTIONS", OPTIONS_SNIPPET),
    ("TRACE", TRACE_SNIPPET),
    ("PATCH", PATCH_SNIPPET),
];

const BYTE_SNIPPETS: [(&str, &str); 3] = [
    ("base64", r#"base64, $1;"#),
    ("file", r#"file, $1;"#),
    ("hex", r#"hex, $1;"#),
];

pub fn completion() -> HashMap<String, ImCompleteCompletionItem> {
    let mut map = HashMap::new();

    const KEYWORDS: [&str; 11] = [
        "Asserts",
        "Query",
        "QueryStringParams",
        "BasicAuth",
        "Form",
        "FormParams",
        "Cookies",
        "Captures",
        "Multipart",
        "MultipartFormData",
        "Options",
    ];
    for keyword in KEYWORDS {
        map.insert(
            keyword.to_owned(),
            ImCompleteCompletionItem::Keyword(keyword.to_owned()),
        );
    }

    const OPTIONS: [&str; 35] = [
        "aws-sigv4",
        "cacert",
        "cert",
        "key",
        "compressed",
        "connect-to",
        "connect-timeout",
        "delay",
        "location",
        "location-trusted",
        "http1.0",
        "http1.1",
        "http2",
        "http3",
        "insecure",
        "ipv4",
        "ipv6",
        "limit-rate",
        "max-redirs",
        "netrc",
        "netrc-file",
        "netrc-optional",
        "output",
        "path-as-is",
        "proxy",
        "repeat",
        "resolve",
        "retry",
        "retry-interval",
        "skip",
        "unix-socket",
        "user",
        "variable",
        "verbose",
        "very-verbose",
    ];
    for option in OPTIONS {
        map.insert(
            option.to_owned(),
            ImCompleteCompletionItem::Keyword(option.to_owned()),
        );
    }

    for method_snippet_tuple in METHOD_SNIPPETS {
        let (method, snippet) = method_snippet_tuple;
        map.insert(
            method.to_owned(),
            ImCompleteCompletionItem::Snippet(method.to_owned(), snippet.to_owned()),
        );
    }

    for byte_snippet_tuple in BYTE_SNIPPETS {
        let (key, snippet) = byte_snippet_tuple;
        map.insert(
            key.to_owned(),
            ImCompleteCompletionItem::Snippet(key.to_owned(), snippet.to_owned()),
        );
    }

    map
}

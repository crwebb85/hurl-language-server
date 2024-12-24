#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Method {
    pub value: String,
    // TODO add a trait to validate if method is valid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ValueString {
    pub value: String,
    // TODO add other fields
    // variables: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Variable {
    VariableName(String),
    FunctionName(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FilterFunction {
    Count,
    DaysAfterNow,
    DaysBeforeNow,
    Decode {
        encoding: InterpolatedString,
        //Official grammar does not have this field but
        //all examples do
    },
    Format {
        fmt: InterpolatedString,
    },
    HtmlEscape,
    HtmlUnescape,
    JsonPath {
        expr: InterpolatedString, //TODO this doesn't match the examples but
                                  //I think this matches the official grammer
    },
    Nth {
        nth: u64,
    },
    Regex {
        value: InterpolatedString,
    },
    Replace {
        old_value: InterpolatedString,
        new_value: InterpolatedString,
    },
    Split {
        sep: InterpolatedString,
    },
    ToDate {
        fmt: InterpolatedString,
    },
    ToFloat,
    ToInt,
    UrlDecode,
    UrlEncode,
    XPath {
        expr: InterpolatedString,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Expr {
    pub variable: Variable,
    pub filters: Vec<FilterFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Template {
    pub expr: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InterpolatedStringPart {
    //I'm using this to represent the type for both
    //value-string-content and key-string-content in
    //the official grammer
    Str(String),
    //I'm using this to represent the type
    //template in the official grammer
    Template(Template),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
//I'm using this to represent the type for both
//value-string and key-string in
//the official grammer
pub struct InterpolatedString {
    pub parts: Vec<InterpolatedStringPart>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyValue {
    pub key: InterpolatedString,
    pub value: InterpolatedString,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Request {
    pub method: Method,
    pub url: ValueString,
    pub header: Vec<KeyValue>,
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
    pub request: Box<Request>,
    pub response: Option<Box<Response>>,
}

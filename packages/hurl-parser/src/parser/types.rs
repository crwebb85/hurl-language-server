use ordered_float::OrderedFloat;
use std::{collections::BTreeMap, hash::Hash};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Method {
    pub value: String,
    // TODO add a trait to validate if method is valid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExprValue {
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
    pub variable: ExprValue,
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
pub struct QueryStringParamsSection {
    pub queries: Vec<KeyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FormParamsSection {
    pub params: Vec<KeyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileValue {
    pub filename: InterpolatedString,
    pub content_type: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileKeyValue {
    pub key: InterpolatedString,
    pub value: FileValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultipartFormParam {
    FileParam(FileKeyValue),
    KeyValueParam(KeyValue),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MultipartFormDataSection {
    pub params: Vec<MultipartFormParam>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CookiesSection {
    pub cookies: Vec<KeyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Capture {
    key: InterpolatedString,
    //TODO add
    // query
    pub filters: Vec<FilterFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CapturesSection {
    pub captures: Vec<Capture>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Assert {
    // TODO
    // pub query: Query,
    pub filters: Vec<FilterFunction>,
    // pub predicate: Predicate
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AssertsSection {
    pub asserts: Vec<Assert>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BasicAuthSection {
    pub key_values: Vec<KeyValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BooleanOption {
    Literal(bool),
    Template(Template),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DurationUnit {
    Millisecond,
    Second,
    Minute,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Duration {
    pub duration: u64,
    pub unit: Option<DurationUnit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DurationOption {
    Literal(Duration),
    Template(Template),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntegerOption {
    Literal(u64), // Hurl uses usize but since hurl is only released in the 64bit mode I will only
    // support 64bit numbers
    Template(Template),
    BigInteger(String), //TODO add a diagnostic error when integer is too large
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VariableValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(OrderedFloat<f64>),
    BigInteger(String),
    String(InterpolatedString),
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariableDefinitionOption {
    pub name: String,
    pub value: VariableValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RequestOption {
    //Boolean Options
    Compressed(BooleanOption),
    Location(BooleanOption),
    LocationTrusted(BooleanOption),
    Http10(BooleanOption),
    Http11(BooleanOption),
    Http2(BooleanOption),
    Http3(BooleanOption),
    Insecure(BooleanOption),
    Ipv4(BooleanOption),
    Ipv6(BooleanOption),
    Netrc(BooleanOption),
    NetrcOptional(BooleanOption),
    PathAsIs(BooleanOption),
    Skip(BooleanOption),
    Verbose(BooleanOption),
    VeryVerbose(BooleanOption),
    //Duration Options
    ConnectTimeout(DurationOption),
    Delay(DurationOption),
    RetryInterval(DurationOption),
    //Integer Options
    LimitRate(IntegerOption),
    MaxRedirs(IntegerOption),
    Repeat(IntegerOption),
    Retry(IntegerOption),
    //Filename options
    Cacert(InterpolatedString),
    Key(InterpolatedString), //TODO this is off spec but official parsers parses this as a filename
    Output(InterpolatedString), //TODO this is off spec but official parsers parses this as a filename
    //Filename password options
    Cert(InterpolatedString),
    //ValueString options
    AwsSigv4(InterpolatedString),
    ConnectTo(InterpolatedString),
    NetrcFile(InterpolatedString),
    Proxy(InterpolatedString),
    Resolve(InterpolatedString),
    UnixSocket(InterpolatedString),
    User(InterpolatedString),
    //Variable options
    Variable(VariableDefinitionOption),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RequestOptionsSection {
    pub options: Vec<RequestOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnknownSection {
    pub section_name: String,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RequestSection {
    BasicAuthSection(BasicAuthSection),
    QueryStringParamsSection(QueryStringParamsSection),
    FormParamsSection(FormParamsSection),
    MultipartFormDataSection(MultipartFormDataSection),
    CookiesSection(CookiesSection),
    OptionsSection(RequestOptionsSection),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Request {
    pub method: Method,
    pub url: InterpolatedString,
    pub header: Vec<KeyValue>, //TODO rename to headers
    pub request_sections: Vec<RequestSection>,
    pub body: Option<Body>,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lt {
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bytes {
    JsonValue(Json),
    // XML,//TODO not yet implemented by hurl cli. Implement it here when that happens
    MultilineString(MultiLineString),
    OneLineString(InterpolatedString),
    OneLineBase64(String),
    OneLineFile(InterpolatedString),
    OneLineHex(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Body {
    pub bytes: Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultiLineString {
    Base64(InterpolatedString),
    Hex(InterpolatedString),
    Json(InterpolatedString),
    Xml(InterpolatedString),
    Graphql(InterpolatedString),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Json {
    Invalid,
    Object(BTreeMap<String, Json>),
    Array(Vec<Json>),
    Str(String),
    InterpolatedString(InterpolatedString),
    Num(OrderedFloat<f64>),
    Bool(bool),
    Null,
}

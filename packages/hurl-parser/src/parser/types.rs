use ordered_float::OrderedFloat;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Method {
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Url {
    Url(InterpolatedString),
    Invalid,
    Missing,
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
        value: Regex,
    },
    Replace {
        old_value: Regex,
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
pub enum CertificateFieldSelector {
    Subject,
    Issuer,
    StartDate,
    ExpireDate,
    SerialNumber,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Query {
    Status,
    Url,
    Header(InterpolatedString),
    Certificate(CertificateFieldSelector),
    Cookie(InterpolatedString),
    Body,
    Xpath(InterpolatedString),
    JsonPath(InterpolatedString),
    Regex(InterpolatedString),
    Variable(InterpolatedString),
    Duration,
    Bytes,
    Sha256,
    Md5,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Capture {
    pub key: InterpolatedString,
    pub query: Query,
    pub filters: Vec<FilterFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CapturesSection {
    pub captures: Vec<Capture>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredicatePrefixOperator {
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredicateValue {
    Boolean(bool),
    MultilineString(MultilineString),
    Null,
    Integer(i64),
    Float(OrderedFloat<f64>),
    BigInteger(String),
    OneLineString(InterpolatedString),
    OneLineBase64(String),
    OneLineFile(InterpolatedString),
    OneLineHex(String),
    QuotedString(InterpolatedString),
    Template(Template),
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredicateFunc {
    Equal { value: PredicateValue },
    NotEqual { value: PredicateValue },
    Greater { value: PredicateValue },
    GreaterOrEqual { value: PredicateValue },
    LessPredicate { value: PredicateValue },
    LessOrEqual { value: PredicateValue },
    StartWith { value: PredicateValue },
    EndWith { value: PredicateValue },
    Contain { value: PredicateValue },
    Match { value: PredicateValue },
    Exists,
    IsEmpty,
    Include { value: PredicateValue },
    IsInteger,
    IsFloat,
    IsBoolean,
    IsString,
    IsCollection,
    IsDate,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Predicate {
    pub prefix: Option<PredicatePrefixOperator>,
    pub function: PredicateFunc,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Assert {
    pub query: Query,
    pub filters: Vec<FilterFunction>,
    pub predicate: Predicate,
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
    pub url: Url,
    pub headers: Vec<KeyValue>, //TODO rename to headers
    pub request_sections: Vec<RequestSection>,
    pub body: Option<Body>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HttpVersion {
    Http1_0,
    Http1_1,
    Http2,
    Http3, //Off spec but is in official parser
    Http,
    HttpUknown(String), //For invalid http versions
}

impl std::fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpVersion::Http1_0 => write!(f, "HTTP/1.0"),
            HttpVersion::Http1_1 => write!(f, "HTTP/1.1"),
            HttpVersion::Http2 => write!(f, "HTTP/2"),
            HttpVersion::Http3 => write!(f, "HTTP/3"),
            HttpVersion::Http => write!(f, "HTTP"),
            HttpVersion::HttpUknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HttpStatus {
    Any,
    Code(u64),
    Invalid, //For when a too large status code is given or otherwise just invalid
    Missing,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResponseSection {
    CapturesSection(CapturesSection),
    AssertsSection(AssertsSection),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Response {
    pub version: HttpVersion,
    pub status: HttpStatus,
    pub headers: Vec<KeyValue>,
    pub response_sections: Vec<ResponseSection>,
    pub body: Option<Body>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    pub request: Box<Request>,
    pub response: Option<Box<Response>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ast {
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Lt {
    pub comment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Bytes {
    JsonValue(Json),
    // XML,//TODO not yet implemented by hurl cli. Implement it here when that happens
    MultilineString(MultilineString),
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
pub enum MultilineStringType {
    Base64,
    Hex,
    Json,
    Xml,
    Graphql,
    Unknown(String), //Used when an unknown type is specified. This will likely result in a
                     //diagnostic error
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MultilineStringAttribute {
    Escape,
    NoVariable,
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MultilineString {
    pub r#type: Option<MultilineStringType>,
    pub attributes: Vec<MultilineStringAttribute>,
    pub content: InterpolatedString,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct JsonKeyValue {
    pub key: InterpolatedString,
    pub value: Json,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Json {
    Invalid,
    Object(Vec<JsonKeyValue>),
    Array(Vec<Json>),
    Str(String),
    InterpolatedString(InterpolatedString),
    Num(String),
    Bool(bool),
    Null,
    Template(Template),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Regex {
    Interpolated(InterpolatedString),
    Literal(String),
}

//TODO replace with custom error type
pub use chumsky::error::Rich;

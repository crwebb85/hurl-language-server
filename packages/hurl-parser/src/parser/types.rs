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
pub struct Template {
    //TODO
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

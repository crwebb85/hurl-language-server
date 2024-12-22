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
pub enum KeyStringPart {
    Str(String),
    Template(Template),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyString {
    pub parts: Vec<KeyStringPart>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyValue {
    pub key: KeyString,
    pub value: String,
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

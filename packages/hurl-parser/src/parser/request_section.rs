use super::filename::filename_parser;
use super::key_value::{key_parser, key_value_parser};
use super::options::option_parser;
use super::primitives::{lt_parser, sp_parser};
use super::types::{
    BasicAuthSection, CookiesSection, FileKeyValue, FileValue, FormParamsSection,
    MultipartFormDataSection, MultipartFormParam, QueryStringParamsSection, RequestOptionsSection,
    RequestSection,
};
use chumsky::prelude::*;

pub fn request_section_parser() -> impl Parser<char, RequestSection, Error = Simple<char>> + Clone {
    let sp = sp_parser();
    let lt = lt_parser();
    let key = key_parser();
    let key_value = key_value_parser();
    let filename = filename_parser();
    let option = option_parser();
    let key_values = key_value.clone().then_ignore(lt.clone()).repeated();

    let basic_auth_section = just("[BasicAuth]")
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map(|(_, auth_key_values)| {
            RequestSection::BasicAuthSection(BasicAuthSection {
                key_values: auth_key_values,
            })
        });

    let query_string_params_section = just("[QueryStringParams]")
        .or(just("[Query]"))
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map(|(_, query_key_values)| {
            RequestSection::QueryStringParamsSection(QueryStringParamsSection {
                queries: query_key_values,
            })
        });

    let form_params_section = just("[FormParams]")
        .or(just("[Form]"))
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map(|(_, form_params)| {
            RequestSection::FormParamsSection(FormParamsSection {
                params: form_params,
            })
        });

    let file_content_type = filter::<_, _, Simple<char>>(|c: &char| {
        c.is_ascii_alphanumeric() || c == &'/' || c == &'+' || c == &'-'
    })
    .repeated()
    .at_least(1)
    .collect::<String>()
    .labelled("file_content_type");

    let file_value = just("file,")
        .then(filename.clone())
        .then_ignore(just(';'))
        .then(file_content_type.or_not())
        .map(|((_, filename), content_type)| FileValue {
            filename,
            content_type,
        });

    let file_param = key
        .clone()
        .then_ignore(sp.clone().repeated()) //TODO: I think this is an offspec sp
        .then_ignore(just(':'))
        .then_ignore(sp.clone().repeated()) //TODO: I think this is an offspec sp
        .then(file_value)
        .map(|(key, value)| MultipartFormParam::FileParam(FileKeyValue { key, value }))
        .labelled("file_key_value");

    let multipart_form_param = file_param.or(key_value.map(MultipartFormParam::KeyValueParam));

    let multipart_form_data_section = just("[MultipartFormData]")
        .or(just("[Multipart]"))
        .then_ignore(lt.clone())
        .then(multipart_form_param.repeated())
        .map(|(_, file_params)| {
            RequestSection::MultipartFormDataSection(MultipartFormDataSection {
                params: file_params,
            })
        });

    let cookies_section = just("[Cookies]")
        .then_ignore(lt.clone())
        .then(key_values.clone())
        .map(|(_, cookies_key_value)| {
            RequestSection::CookiesSection(CookiesSection {
                cookies: cookies_key_value,
            })
        });

    let options_section = just("[Options]")
        .then_ignore(lt.clone())
        .then(option.repeated())
        .map(|(_, options)| RequestSection::OptionsSection(RequestOptionsSection { options }));

    let request_section = basic_auth_section
        .or(query_string_params_section)
        .or(form_params_section)
        .or(multipart_form_data_section)
        .or(cookies_section)
        .or(options_section);
    // TODO and an unknown section for error handling
    // .or(unknown_section);

    request_section
}

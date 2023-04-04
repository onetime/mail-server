use std::collections::HashMap;

use crate::{
    error::{
        method::MethodError,
        request::{RequestError, RequestLimitError},
    },
    method::{
        changes::ChangesRequest,
        copy::{CopyBlobRequest, CopyRequest},
        get::GetRequest,
        import::ImportEmailRequest,
        parse::ParseEmailRequest,
        query::QueryRequest,
        query_changes::QueryChangesRequest,
        set::SetRequest,
        validate::ValidateSieveScriptRequest,
    },
    parser::{json::Parser, Error, Ignore, JsonObjectParser, Token},
    types::id::Id,
};

use super::{
    capability::Capability,
    echo::Echo,
    method::{MethodFunction, MethodName, MethodObject},
    Call, Request, RequestMethod,
};

impl Request {
    pub fn parse(json: &[u8], max_calls: usize, max_size: usize) -> Result<Self, RequestError> {
        if json.len() <= max_size {
            let mut request = Request {
                using: 0,
                method_calls: Vec::new(),
                created_ids: None,
            };
            let mut found_valid_keys = false;
            let mut parser = Parser::new(json);
            parser.next_token::<String>()?.assert(Token::DictStart)?;
            while {
                match parser.next_dict_key::<u128>()? {
                    0x676e_6973_75 => {
                        found_valid_keys = true;
                        parser.next_token::<Ignore>()?.assert(Token::ArrayStart)?;
                        while {
                            request.using |=
                                parser.next_token::<Capability>()?.unwrap_string("using")? as u32;
                            !parser.is_array_end()?
                        } {}
                    }
                    0x736c_6c61_4364_6f68_7465_6d => {
                        found_valid_keys = true;

                        parser
                            .next_token::<Ignore>()?
                            .assert_jmap(Token::ArrayStart)?;
                        while {
                            if request.method_calls.len() < max_calls {
                                parser
                                    .next_token::<Ignore>()?
                                    .assert_jmap(Token::ArrayStart)?;
                                let method = match parser.next_token::<MethodName>() {
                                    Ok(Token::String(method)) => method,
                                    Ok(_) => {
                                        return Err(RequestError::not_request(
                                            "Invalid JMAP request",
                                        ));
                                    }
                                    Err(Error::Method(MethodError::InvalidArguments(_))) => {
                                        MethodName::unknown_method()
                                    }
                                    Err(err) => {
                                        return Err(err.into());
                                    }
                                };
                                parser.next_token::<Ignore>()?.assert_jmap(Token::Comma)?;
                                parser.ctx = method.obj;
                                let start_depth_array = parser.depth_array;
                                let start_depth_dict = parser.depth_dict;

                                let method = match (&method.fnc, &method.obj) {
                                    (MethodFunction::Get, _) => {
                                        GetRequest::parse(&mut parser).map(RequestMethod::Get)
                                    }
                                    (MethodFunction::Query, _) => {
                                        QueryRequest::parse(&mut parser).map(RequestMethod::Query)
                                    }
                                    (MethodFunction::Set, _) => {
                                        SetRequest::parse(&mut parser).map(RequestMethod::Set)
                                    }
                                    (MethodFunction::Changes, _) => {
                                        ChangesRequest::parse(&mut parser)
                                            .map(RequestMethod::Changes)
                                    }
                                    (MethodFunction::QueryChanges, _) => {
                                        QueryChangesRequest::parse(&mut parser)
                                            .map(RequestMethod::QueryChanges)
                                    }
                                    (MethodFunction::Copy, MethodObject::Email) => {
                                        CopyRequest::parse(&mut parser).map(RequestMethod::Copy)
                                    }
                                    (MethodFunction::Copy, MethodObject::Blob) => {
                                        CopyBlobRequest::parse(&mut parser)
                                            .map(RequestMethod::CopyBlob)
                                    }
                                    (MethodFunction::Import, MethodObject::Email) => {
                                        ImportEmailRequest::parse(&mut parser)
                                            .map(RequestMethod::ImportEmail)
                                    }
                                    (MethodFunction::Parse, MethodObject::Email) => {
                                        ParseEmailRequest::parse(&mut parser)
                                            .map(RequestMethod::ParseEmail)
                                    }
                                    (MethodFunction::Validate, MethodObject::SieveScript) => {
                                        ValidateSieveScriptRequest::parse(&mut parser)
                                            .map(RequestMethod::ValidateScript)
                                    }
                                    (MethodFunction::Echo, MethodObject::Core) => {
                                        Echo::parse(&mut parser).map(RequestMethod::Echo)
                                    }
                                    _ => Err(Error::Method(MethodError::UnknownMethod(
                                        method.to_string(),
                                    ))),
                                };

                                let method = match method {
                                    Ok(method) => method,
                                    Err(Error::Method(err)) => {
                                        parser.skip_token(start_depth_array, start_depth_dict)?;
                                        RequestMethod::Error(err)
                                    }
                                    Err(err) => {
                                        return Err(err.into());
                                    }
                                };

                                parser.next_token::<Ignore>()?.assert_jmap(Token::Comma)?;
                                let id = parser.next_token::<String>()?.unwrap_string("")?;
                                parser
                                    .next_token::<Ignore>()?
                                    .assert_jmap(Token::ArrayEnd)?;
                                request.method_calls.push(Call { id, method });
                            } else {
                                return Err(RequestError::limit(RequestLimitError::CallsIn));
                            }
                            !parser.is_array_end()?
                        } {}
                    }
                    0x7364_4964_6574_6165_7263 => {
                        found_valid_keys = true;
                        let mut created_ids = HashMap::new();
                        parser.next_token::<Ignore>()?.assert(Token::DictStart)?;
                        while {
                            created_ids.insert(
                                parser.next_dict_key::<String>()?,
                                parser.next_token::<Id>()?.unwrap_string("createdIds")?,
                            );
                            !parser.is_dict_end()?
                        } {}
                        request.created_ids = Some(created_ids);
                    }
                    _ => {
                        parser.skip_token(parser.depth_array, parser.depth_dict)?;
                    }
                }

                !parser.is_dict_end()?
            } {}

            if found_valid_keys {
                Ok(request)
            } else {
                Err(RequestError::not_request("Invalid JMAP request"))
            }
        } else {
            Err(RequestError::limit(RequestLimitError::Size))
        }
    }
}

impl From<Error> for RequestError {
    fn from(value: Error) -> Self {
        match value {
            Error::Request(err) => err,
            Error::Method(err) => RequestError::not_request(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::request::Request;

    const TEST: &str = r#"
    {
        "using": [ "urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail" ],
        "methodCalls": [
          [ "method1", {
            "arg1": "arg1data",
            "arg2": "arg2data"
          }, "c1" ],
          [ "Core/echo", {
            "hello": true,
            "high": 5
          }, "c2" ],
          [ "method3", {"hello": [{"a": {"b": true}}]}, "c3" ]
        ],
        "createdIds": {
            "c1": "m1",
            "c2": "m2"
        }
      }
    "#;

    #[test]
    fn parse_request() {
        println!("{:?}", Request::parse(TEST.as_bytes(), 10, 1024));
    }
}

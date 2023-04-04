use crate::{
    error::method::MethodError,
    object::{email, Object},
    parser::{json::Parser, Error, JsonObjectParser, Token},
    request::{
        method::MethodObject,
        reference::{MaybeReference, ResultReference},
        RequestProperty, RequestPropertyParser,
    },
    types::{id::Id, property::Property, state::State, value::Value},
};

#[derive(Debug, Clone)]
pub struct GetRequest {
    pub account_id: Id,
    pub ids: Option<MaybeReference<Vec<Id>, ResultReference>>,
    pub properties: Option<MaybeReference<Vec<Property>, ResultReference>>,
    pub arguments: RequestArguments,
}

#[derive(Debug, Clone)]
pub enum RequestArguments {
    Email(email::GetArguments),
    Mailbox,
    Thread,
    Identity,
    EmailSubmission,
    PushSubscription,
    SieveScript,
    VacationResponse,
    Principal,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GetResponse {
    #[serde(rename = "accountId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<Id>,

    pub state: State,

    pub list: Vec<Object<Value>>,

    #[serde(rename = "notFound")]
    pub not_found: Vec<Id>,
}

impl JsonObjectParser for GetRequest {
    fn parse(parser: &mut Parser<'_>) -> crate::parser::Result<Self>
    where
        Self: Sized,
    {
        let mut request = GetRequest {
            arguments: match &parser.ctx {
                MethodObject::Email => RequestArguments::Email(Default::default()),
                MethodObject::Mailbox => RequestArguments::Mailbox,
                MethodObject::Thread => RequestArguments::Thread,
                MethodObject::Identity => RequestArguments::Identity,
                MethodObject::EmailSubmission => RequestArguments::EmailSubmission,
                MethodObject::PushSubscription => RequestArguments::PushSubscription,
                MethodObject::SieveScript => RequestArguments::SieveScript,
                MethodObject::VacationResponse => RequestArguments::VacationResponse,
                MethodObject::Principal => RequestArguments::Principal,
                _ => {
                    return Err(Error::Method(MethodError::UnknownMethod(format!(
                        "{}/get",
                        parser.ctx
                    ))))
                }
            },
            account_id: Id::default(),
            ids: None,
            properties: None,
        };

        parser
            .next_token::<String>()?
            .assert_jmap(Token::DictStart)?;

        while {
            let property = parser.next_dict_key::<RequestProperty>()?;
            match &property.hash[0] {
                0x6449_746e_756f_6363_61 if !property.is_ref => {
                    request.account_id = parser.next_token::<Id>()?.unwrap_string("accountId")?;
                }
                0x7364_69 => {
                    request.ids = if !property.is_ref {
                        <Option<Vec<Id>>>::parse(parser)?.map(MaybeReference::Value)
                    } else {
                        Some(MaybeReference::Reference(ResultReference::parse(parser)?))
                    };
                }
                0x7365_6974_7265_706f_7270 => {
                    request.properties = if !property.is_ref {
                        <Option<Vec<Property>>>::parse(parser)?.map(MaybeReference::Value)
                    } else {
                        Some(MaybeReference::Reference(ResultReference::parse(parser)?))
                    };
                }
                _ => {
                    if !request.arguments.parse(parser, property)? {
                        parser.skip_token(parser.depth_array, parser.depth_dict)?;
                    }
                }
            }

            !parser.is_dict_end()?
        } {}

        Ok(request)
    }
}

impl RequestPropertyParser for RequestArguments {
    fn parse(
        &mut self,
        parser: &mut Parser,
        property: RequestProperty,
    ) -> crate::parser::Result<bool> {
        if let RequestArguments::Email(arguments) = self {
            arguments.parse(parser, property)
        } else {
            Ok(false)
        }
    }
}

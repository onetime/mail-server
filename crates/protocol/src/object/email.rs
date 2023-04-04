use crate::{
    parser::{json::Parser, Ignore, JsonObjectParser},
    request::{RequestProperty, RequestPropertyParser},
    types::property::Property,
};

#[derive(Debug, Clone, Default)]
pub struct GetArguments {
    pub body_properties: Option<Vec<Property>>,
    pub fetch_text_body_values: Option<bool>,
    pub fetch_html_body_values: Option<bool>,
    pub fetch_all_body_values: Option<bool>,
    pub max_body_value_bytes: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct QueryArguments {
    collapse_threads: Option<bool>,
}

impl RequestPropertyParser for GetArguments {
    fn parse(
        &mut self,
        parser: &mut Parser,
        property: RequestProperty,
    ) -> crate::parser::Result<bool> {
        match (&property.hash[0], &property.hash[1]) {
            (0x7365_6974_7265_706f_7250_7964_6f62, _) => {
                self.body_properties = <Option<Vec<Property>>>::parse(parser)?;
            }
            (0x6c61_5679_646f_4274_7865_5468_6374_6566, 0x7365_75) => {
                self.fetch_text_body_values = parser
                    .next_token::<Ignore>()?
                    .unwrap_bool_or_null("fetchTextBodyValues")?;
            }
            (0x6c61_5679_646f_424c_4d54_4868_6374_6566, 0x7365_75) => {
                self.fetch_html_body_values = parser
                    .next_token::<Ignore>()?
                    .unwrap_bool_or_null("fetchHTMLBodyValues")?;
            }
            (0x756c_6156_7964_6f42_6c6c_4168_6374_6566, 0x7365) => {
                self.fetch_all_body_values = parser
                    .next_token::<Ignore>()?
                    .unwrap_bool_or_null("fetchAllBodyValues")?;
            }
            (0x6574_7942_6575_6c61_5679_646f_4278_616d, 0x73) => {
                self.max_body_value_bytes = parser
                    .next_token::<Ignore>()?
                    .unwrap_usize_or_null("maxBodyValueBytes")?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}

impl RequestPropertyParser for QueryArguments {
    fn parse(
        &mut self,
        parser: &mut Parser,
        property: RequestProperty,
    ) -> crate::parser::Result<bool> {
        if property.hash[0] == 0x7364_6165_7268_5465_7370_616c_6c6f_63 {
            self.collapse_threads = parser
                .next_token::<Ignore>()?
                .unwrap_bool_or_null("collapseThreads")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

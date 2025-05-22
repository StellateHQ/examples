use gql_query::ast::{ASTContext, Document, ParseNode, PrintNode};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RawGraphQLInput {
    query: String,
    operation_name: Option<String>,
    variables: Option<Map<String, Value>>,
    extensions: Option<Map<String, Value>>,
}

pub struct GraphQLInput<'a> {
    pub ctx: &'a ASTContext,
    pub document: &'a Document<'a>,
    pub operation_name: Option<String>,
    pub variables: Option<Map<String, Value>>,
    pub extensions: Option<Map<String, Value>>,
}

impl<'a> GraphQLInput<'a> {
    pub fn new(ctx: &'a ASTContext, body: &[u8]) -> anyhow::Result<Self> {
        let raw = serde_json::from_slice::<RawGraphQLInput>(body)?;

        let query = Document::parse(&ctx, raw.query)?;

        Ok(Self {
            ctx,
            document: query,
            operation_name: raw.operation_name,
            variables: raw.variables,
            extensions: raw.extensions,
        })
    }

    pub fn query(&self) -> String {
        self.document.print()
    }

    pub fn to_body(&self) -> anyhow::Result<String> {
        let body = serde_json::to_string(&RawGraphQLInput {
            query: self.query(),
            operation_name: self.operation_name.clone(),
            variables: self.variables.clone(),
            extensions: self.extensions.clone(),
        })?;
        Ok(body)
    }
}

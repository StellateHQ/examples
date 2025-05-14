use std::collections::HashMap;

use crate::graphql::GraphQLInput;
use anyhow::{anyhow, Error, Result};
use gql_query::{
    ast::Field,
    visit::{VisitFlow, VisitInfo, VisitNode, Visitor},
};
use json_dotpath::DotPaths;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    #[serde(
        default,
        deserialize_with = "deserialize_some",
        skip_serializing_if = "Option::is_none"
    )]
    pub data: Option<Option<Map<String, Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Map<String, Value>>,
}

/// Any value that is present is considered `Some` value, including `null`. This allows us to differentiate between:
/// - missing properties (`None`)
/// - properties with `null` as value (`Some(None)`)
/// - properties with non-null values (`Some(Some(T))`)
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

pub struct PartialQuery<'a> {
    pub input: GraphQLInput<'a>,
    pub response: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct StellateExtensions {
    partial_queries: Vec<PartialQueryExtensions>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct PartialQueryExtensions {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_extensions: Option<Map<String, Value>>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct StellateResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Option<Map<String, Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<Value>>,

    extensions: HashMap<String, StellateExtensions>,
}

pub fn merge_responses(input: &GraphQLInput, parts: Vec<PartialQuery>) -> Result<String> {
    let mut some_data_exists = false;
    let mut all_data: Vec<Map<String, Value>> = vec![];
    let mut errors: Vec<Value> = vec![];
    let mut extensions = StellateExtensions::default();

    for part in parts {
        let response = serde_json::from_str::<Response>(&part.response)
            .map_err(|_| anyhow!("Cannot parse response: {}", part.response))?;

        match response.data {
            Some(Some(data)) => {
                some_data_exists = true;
                all_data.push(data);
            }
            Some(None) => some_data_exists = true,
            None => {}
        }

        match response.errors {
            Some(err) => errors.extend(err),
            _ => {}
        }

        extensions.partial_queries.push(PartialQueryExtensions {
            query: part.input.query(),
            response_extensions: response.extensions,
        })
    }

    let errors = if errors.len() == 0 {
        None
    } else {
        Some(errors)
    };
    let extensions = HashMap::from([(String::from("stellate"), extensions)]);

    if all_data.len() == 0 {
        let response = StellateResponse {
            data: if some_data_exists { Some(None) } else { None },
            errors,
            extensions,
        };
        return Ok(serde_json::to_string(&response)?);
    }

    let mut merge_visitor = MergeResponseVisitor::new(all_data);
    input.document.visit(&(), &mut merge_visitor);
    if let Some(error) = merge_visitor.error {
        return Err(error);
    }

    let response = StellateResponse {
        data: Some(Some(merge_visitor.data)),
        errors,
        extensions,
    };
    return Ok(serde_json::to_string(&response)?);
}

struct MergeResponseVisitor<'a> {
    all_data: Vec<Map<String, Value>>,
    data: Map<String, Value>,
    path: Vec<&'a str>,
    error: Option<Error>,
}

impl<'a> MergeResponseVisitor<'a> {
    fn new(all_data: Vec<Map<String, Value>>) -> Self {
        Self {
            all_data,
            data: Map::default(),
            path: vec![],
            error: None,
        }
    }
}

impl<'a> Visitor<'a> for MergeResponseVisitor<'a> {
    fn enter_field(&mut self, _ctx: &(), field: &'a Field<'a>, _info: &VisitInfo) -> VisitFlow {
        self.path.push(field.name);

        let dot_path = self.path.join(".");
        let mut value = Value::default();
        for partial_data in self.all_data.iter() {
            if let Ok(Some(v)) = partial_data.dot_get::<Value>(&dot_path) {
                if v.is_null() || v.is_object() || v.is_array() {
                    continue;
                }

                match &value {
                    Value::Null => value = v,
                    value @ Value::Bool(_)
                    | value @ Value::Number(_)
                    | value @ Value::String(_) => {
                        if value != &v {
                            self.error = Some(anyhow!(
                                "Path {dot_path} contains different values: {value:?}, {v:?}"
                            ));
                            return VisitFlow::Break;
                        }
                    }
                    _ => {}
                }
            }
        }

        match self.data.dot_set(&dot_path, value) {
            Ok(_) => VisitFlow::Next,
            Err(err) => {
                self.error = Some(anyhow!("Failed to set path {dot_path}: {err}"));
                VisitFlow::Break
            }
        }
    }

    fn leave_field(&mut self, _ctx: &(), _field: &'a Field<'a>, _info: &VisitInfo) -> VisitFlow {
        self.path.pop();
        VisitFlow::Next
    }
}

#[cfg(test)]
mod tests {
    use crate::{graphql::GraphQLInput, merge::response::PartialQuery};
    use gql_query::ast::ASTContext;
    use serde_json::json;

    use super::merge_responses;

    #[test]
    fn top_level_fields() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                lowMaxAge
                highMaxAge
                noMaxAge
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ highMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(&json!({ "data": { "highMaxAge": 200 } })).unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ lowMagAge noMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b =
            serde_json::to_string(&json!({ "data": { "lowMaxAge": 100, "noMaxAge": null } }))
                .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "lowMaxAge": 100,
            "highMaxAge": 200,
            "noMaxAge": null
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn nested_fields() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                nested {
                  lowMaxAge
                  highMaxAge
                  noMaxAge
                }
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ nested { highMaxAge } }" })).unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a =
            serde_json::to_string(&json!({ "data": { "nested": { "highMaxAge": 200 } } })).unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ nested { lowMagAge noMaxAge } }" })).unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(
            &json!({ "data": { "nested": { "lowMaxAge": 100, "noMaxAge": null } } }),
        )
        .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "nested": {
              "lowMaxAge": 100,
              "highMaxAge": 200,
              "noMaxAge": null
            }
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn deeply_nested_fields() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                nested {
                  nested {
                    nested {
                      lowMaxAge
                      highMaxAge
                      noMaxAge
                    }
                  }
                }
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                &json!({ "query": "{ nested { nested { nested { highMaxAge } } } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(
            &json!({ "data": { "nested": { "nested": { "nested": { "highMaxAge": 200 } } } } }),
        )
        .unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                &json!({ "query": "{ nested { nested { nested { lowMagAge noMaxAge } } } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(
          &json!({ "data": { "nested": { "nested": { "nested": { "lowMaxAge": 100, "noMaxAge": null } } } } }),
        )
        .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "nested": {
              "nested": {
                "nested": {
                  "lowMaxAge": 100,
                  "highMaxAge": 200,
                  "noMaxAge": null
                }
              }
            }
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn combined() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                highMaxAge
                lowMaxAge
                zeroMaxAge
                noMaxAge
                nested {
                  highMaxAge
                  lowMaxAge
                  zeroMaxAge
                  noMaxAge
                }
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ highMaxAge nested { highMaxAge } }" }))
                .unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(
            &json!({ "data": { "highMaxAge": 200, "nested": { "highMaxAge": 200 } } }),
        )
        .unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ lowMaxAge nested { lowMaxAge } }" })).unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(
            &json!({ "data": { "lowMaxAge": 100, "nested": { "lowMaxAge": 100 } } }),
        )
        .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let input_c = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                &json!({ "query": "{ zeroMaxAge noMaxAge nested { zeroMaxAge noMaxAge } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_c = input_c.query();
        let response_c = serde_json::to_string(
            &json!({ "data": { "zeroMaxAge": 0, "noMaxAge": null, "nested": { "zeroMaxAge": 0, "noMaxAge": null } } }),
        )
        .unwrap();
        let partial_query_c = PartialQuery {
            input: input_c,
            response: response_c,
        };

        let response = merge_responses(
            &input,
            vec![partial_query_a, partial_query_b, partial_query_c],
        )
        .unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "highMaxAge": 200,
            "lowMaxAge": 100,
            "zeroMaxAge": 0,
            "noMaxAge": null,
            "nested": {
              "highMaxAge": 200,
              "lowMaxAge": 100,
              "zeroMaxAge": 0,
              "noMaxAge": null
            }
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b },
                { "query": query_c }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn top_level_fields_in_fragment() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                ...Stuff
              }

              fragment Stuff on Query {
                lowMaxAge
                highMaxAge
                noMaxAge
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ ... on Query { highMaxAge } }" })).unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(&json!({ "data": { "highMaxAge": 200 } })).unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ ... on Query { lowMaxAge noMaxAge } }" }))
                .unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b =
            serde_json::to_string(&json!({ "data": { "lowMaxAge": 100, "noMaxAge": null } }))
                .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "lowMaxAge": 100,
            "highMaxAge": 200,
            "noMaxAge": null
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn nested_fields_in_fragment() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                nested {
                  ...Stuff
                }
              }

              fragment Stuff on Query {
                nested {
                  lowMaxAge
                  highMaxAge
                  noMaxAge
                }
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                &json!({ "query": "{ nested { ... on Query { nested { highMaxAge } } } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(
            &json!({ "data": { "nested": { "nested": { "highMaxAge": 200 } } } }),
        )
        .unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ nested { ... on Query { nested { lowMaxAge noMaxAge } } } }" }))
                .unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(
            &json!({ "data": { "nested": { "nested": { "lowMaxAge": 100, "noMaxAge": null } } } }),
        )
        .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "nested": {
              "nested": {
                "lowMaxAge": 100,
                "highMaxAge": 200,
                "noMaxAge": null
              }
            }
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn fragment_with_sibling_fields() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                highMaxAge
                lowMaxAge
                zeroMaxAge
                noMaxAge
                ...Stuff
              }

              fragment Stuff on Query {
                highMaxAge
                lowMaxAge
                zeroMaxAge
                noMaxAge
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ highMaxAge ... on Query { highMaxAge } }" }))
                .unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(&json!({ "data": { "highMaxAge": 200 } })).unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ lowMaxAge ... on Query { lowMaxAge } }" }))
                .unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(&json!({ "data": { "lowMaxAge": 100 } })).unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let input_c = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                &json!({ "query": "{ zeroMaxAge noMaxAge ... on Query { zeroMaxAge noMaxAge } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_c = input_c.query();
        let response_c =
            serde_json::to_string(&json!({ "data": { "zeroMaxAge": 0, "noMaxAge": null } }))
                .unwrap();
        let partial_query_c = PartialQuery {
            input: input_c,
            response: response_c,
        };

        let response = merge_responses(
            &input,
            vec![partial_query_a, partial_query_b, partial_query_c],
        )
        .unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "highMaxAge": 200,
            "lowMaxAge": 100,
            "zeroMaxAge": 0,
            "noMaxAge": null
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b },
                { "query": query_c }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn inline_fragments() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                node(id: 42) {
                  id
                  ... on Todo {
                    text
                    authors {
                      name
                    }
                  }
                }
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                // TODO: We want to also preserve the "id" field inside the "node" because it's a key-field.
                &json!({ "query": "{ node(id: 42) { ... on Todo { authors { name } } } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(
            &json!({ "data": { "node": { "authors": { "name": "Thomas" } } } }),
        )
        .unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(
                &json!({ "query": "{ node(id: 42) { id ... on Todo { text } } }" }),
            )
            .unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(
            &json!({ "data": { "node": { "id": "42", "text": "Get milk" } } }),
        )
        .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "node": {
              "id": "42",
              "text": "Get milk",
              "authors": [{ "name": "Thomas" }]
            }
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn errors() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                lowMaxAge
                highMaxAge
                noMaxAge
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ highMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a =
            serde_json::to_string(&json!({ "errors": [{ "message": "oopsie" }] })).unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ lowMagAge noMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b =
            serde_json::to_string(&json!({ "errors": [{ "message": "doopsie" }] })).unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "errors": [
            { "message": "oopsie" },
            { "message": "doopsie" }
          ],
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn null_data() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                lowMaxAge
                highMaxAge
                noMaxAge
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ highMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(&json!({ "data": null })).unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ lowMagAge noMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(&json!({ "data": null })).unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": null,
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a },
                { "query": query_b }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }

    #[test]
    fn extensions() {
        let ctx = ASTContext::new();

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query {
                lowMaxAge
                highMaxAge
                noMaxAge
              }
            "#,
        }))
        .unwrap();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let input_a = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ highMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_a = input_a.query();
        let response_a = serde_json::to_string(&json!({
          "data": { "highMaxAge": 200 },
          "extensions": { "responseA": "foo" }
        }))
        .unwrap();
        let partial_query_a = PartialQuery {
            input: input_a,
            response: response_a,
        };

        let input_b = GraphQLInput::new(
            &ctx,
            &serde_json::to_vec(&json!({ "query": "{ lowMagAge noMaxAge }" })).unwrap(),
        )
        .unwrap();
        let query_b = input_b.query();
        let response_b = serde_json::to_string(&json!({
          "data": { "lowMaxAge": 100, "noMaxAge": null },
          "extensions": { "responseB": "bar" }
        }))
        .unwrap();
        let partial_query_b = PartialQuery {
            input: input_b,
            response: response_b,
        };

        let response = merge_responses(&input, vec![partial_query_a, partial_query_b]).unwrap();
        let expected = serde_json::to_string(&json!({
          "data": {
            "lowMaxAge": 100,
            "highMaxAge": 200,
            "noMaxAge": null
          },
          "extensions": {
            "stellate": {
              "partialQueries": [
                { "query": query_a, "responseExtensions": { "responseA": "foo" } },
                { "query": query_b, "responseExtensions": { "responseB": "bar" } }
              ]
            }
          }
        }))
        .unwrap();
        assert_eq!(response, expected);
    }
}

mod inline_fragments;
mod query_splitter;
mod selection_set_extractor;
mod selection_set_replacer;

use self::{
    inline_fragments::InlineFragments, query_splitter::QuerySplitter,
    selection_set_extractor::SelectionSetExtractor, selection_set_replacer::SelectionSetReplacer,
};
use crate::{graphql::GraphQLInput, manifest::Manifest, merge::document::merge_documents};
use anyhow::anyhow;
use gql_query::{
    ast::Document,
    schema::Schema,
    visit::{FoldNode, VisitNode},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DocumentSplit<'a> {
    pub document: &'a Document<'a>,
    pub max_age: Option<u64>,
}

pub fn split<'a>(
    input: &'a GraphQLInput<'a>,
    schema: &'a Schema<'a>,
    manifest: &'a Manifest,
) -> anyhow::Result<HashMap<Option<u64>, &'a Document<'a>>> {
    input.document.inline_fragments(
        &input.ctx,
        input.operation_name.as_ref().map(|s| s.as_str()),
    )?;
    let mut documents_to_split = vec![DocumentSplit {
        document: &input.document,
        max_age: None,
    }];
    let mut document_splits = HashMap::<Option<u64>, &'a Document<'a>>::new();

    while documents_to_split.len() > 0 {
        let document_split = documents_to_split.pop().unwrap();

        let mut splitter = QuerySplitter::new(schema, manifest, &input.operation_name);
        document_split.document.visit(&input.ctx, &mut splitter);
        let result = match splitter.result {
            Some(Ok(r)) => r,
            Some(Err(e)) => return Err(e),
            None => {
                // The document can't be split into more parts
                match document_splits.get(&document_split.max_age) {
                    Some(document) => {
                        let operation_name = input.operation_name.as_ref().map(|s| s.as_str());
                        let merged = merge_documents(
                            input.ctx,
                            document,
                            document_split.document,
                            operation_name,
                        )
                        .ok_or(anyhow!(
                            "Failed to merge queries for operation name {operation_name:?}",
                        ))?;
                        document_splits.insert(document_split.max_age, merged);
                    }
                    None => {
                        document_splits.insert(document_split.max_age, document_split.document);
                    }
                };
                continue;
            }
        };

        let mut replacer = SelectionSetReplacer::new(result.path.clone(), result.selection_set);
        documents_to_split.push(DocumentSplit {
            document: document_split.document.fold(input.ctx, &mut replacer)?,
            max_age: result.max_age,
        });

        for (max_age, selection_set) in result.splits {
            let mut replacer = SelectionSetExtractor::new(result.path.clone(), selection_set);
            documents_to_split.push(DocumentSplit {
                document: document_split.document.fold(input.ctx, &mut replacer)?,
                max_age: Some(max_age),
            });
        }
    }

    Ok(document_splits)
}

#[cfg(test)]
mod tests {
    use crate::schema::{MANIFEST, SCHEMA};

    use super::*;
    use gql_query::{
        ast::{ASTContext, PrintNode},
        schema::{BuildClientSchema, IntrospectionQuery},
    };
    use serde_json::json;
    use textwrap::dedent;

    #[test]
    fn top_level_fields() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);

        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    highMaxAge
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    lowMaxAge
                    noMaxAge
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn nested_fields() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);

        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      highMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      lowMaxAge
                      noMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn deeply_nested_fields() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);

        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      nested {
                        nested {
                          highMaxAge
                        }
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      nested {
                        nested {
                          lowMaxAge
                          noMaxAge
                        }
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn combined() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 3);

        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    highMaxAge
                    nested {
                      highMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    lowMaxAge
                    nested {
                      lowMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(0)).unwrap().print(),
            dedent(
                r#"
                  {
                    zeroMaxAge
                    noMaxAge
                    nested {
                      zeroMaxAge
                      noMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn multiple_operations() {
        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query A {
                lowMaxAge
                highMaxAge
                noMaxAge
              }

              query B {
                __typename
              }
            "#,
            "operationName": "A"
        }))
        .unwrap();
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);
        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  query A {
                    highMaxAge
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  query A {
                    lowMaxAge
                    noMaxAge
                  }
                "#,
            )
            .trim(),
        );

        let body_bytes = serde_json::to_vec(&json!({
            "query": r#"
              query A {
                lowMaxAge
                highMaxAge
                noMaxAge
              }

              query B {
                __typename
              }
            "#,
            "operationName": "B"
        }))
        .unwrap();
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(
            documents.get(&None).unwrap().print(),
            dedent(
                r#"
                  query B {
                    __typename
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn top_level_fields_in_fragment() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);
        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    ... on Query {
                      highMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    ... on Query {
                      lowMaxAge
                      noMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn nested_fields_in_fragment() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);
        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      ... on Query {
                        nested {
                          highMaxAge
                        }
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      ... on Query {
                        nested {
                          lowMaxAge
                          noMaxAge
                        }
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn fragment_with_sibling_fields() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 3);
        assert_eq!(
            documents.get(&Some(200)).unwrap().print(),
            dedent(
                r#"
                  {
                    highMaxAge
                    ... on Query {
                      highMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );
        assert_eq!(
            documents.get(&Some(100)).unwrap().print(),
            dedent(
                r#"
                  {
                    lowMaxAge
                    ... on Query {
                      lowMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );
        assert_eq!(
            documents.get(&Some(0)).unwrap().print(),
            dedent(
                r#"
                  {
                    zeroMaxAge
                    noMaxAge
                    ... on Query {
                      zeroMaxAge
                      noMaxAge
                    }
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn inline_fragments() {
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
        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);
        assert_eq!(
            documents.get(&Some(900)).unwrap().print(),
            // TODO: We want to also preserve the "id" field inside the "node" because it's a key-field.
            dedent(
                r#"
                  {
                    node(id: 42) {
                      ... on Todo {
                        authors {
                          name
                        }
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );
        assert_eq!(
            documents.get(&Some(600)).unwrap().print(),
            dedent(
                r#"
                  {
                    node(id: 42) {
                      id
                      ... on Todo {
                        text
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );
    }
}

#[cfg(test)]
mod medium_test {
    use super::*;
    use crate::{graphql::GraphQLInput, manifest::Manifest, schema::SCHEMA};
    use gql_query::{
        ast::{ASTContext, PrintNode},
        schema::{BuildClientSchema, IntrospectionQuery},
    };
    use serde_json::json;
    use textwrap::dedent;

    #[test]
    fn medium() {
        let body_bytes = serde_json::to_vec(&json!({
          "query": r#"
            query PostPageQuery($postId: ID!) {
              medium {
                post(id: $postId) {
                  id
                  creator {
                    id
                    name
                  }
                  content {
                    bodyModel {
                      paragraphs {
                        id
                        type
                        text
                      }
                    }
                  }
                }
              }
            }
          "#,
          "operationName": "PostPageQuery"
        }))
        .unwrap();

        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(
            r#"
              {
                "cacheConfig": {
                  "Medium": {
                    "cacheControl": { "maxAge": 0, "swr": null, "scope": null },
                    "keyFields": null,
                    "fields": {}
                  },
                  "Medium_Post": {
                    "cacheControl": null,
                    "keyFields": null,
                    "fields": {
                      "content": { "cacheControl": { "maxAge": 3600, "swr": null, "scope": null } }
                    }
                  }
                }
              }
            "#,
        )
        .unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);

        assert_eq!(
            documents.get(&Some(3600)).unwrap().print(),
            dedent(
                r#"
                  query PostPageQuery($postId: ID!) {
                    medium {
                      post(id: $postId) {
                        content {
                          bodyModel {
                            paragraphs {
                              id
                              type
                              text
                            }
                          }
                        }
                        id
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(0)).unwrap().print(),
            dedent(
                r#"
                  query PostPageQuery($postId: ID!) {
                    medium {
                      post(id: $postId) {
                        creator {
                          id
                          name
                        }
                        id
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );
    }
}

#[cfg(test)]
mod puma_test {
    use super::*;
    use crate::{graphql::GraphQLInput, manifest::Manifest, schema::SCHEMA};
    use gql_query::{
        ast::{ASTContext, PrintNode},
        schema::{BuildClientSchema, IntrospectionQuery},
    };
    use serde_json::json;
    use textwrap::dedent;

    #[test]
    fn puma() {
        let body_bytes = serde_json::to_vec(&json!({
          "query": r#"
            {
              puma {
                products {
                  id
                  name
                  price
                }
              }
            }
          "#
        }))
        .unwrap();

        let ctx = ASTContext::new();
        let input = GraphQLInput::new(&ctx, &body_bytes).unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(
            r#"
              {
                "cacheConfig": {
                  "Puma": {
                    "cacheControl": null,
                    "keyFields": null,
                    "fields": {
                      "products": { "cacheControl": { "maxAge": 3600, "swr": null, "scope": null } }
                    }
                  },
                  "Puma_Product": {
                    "cacheControl": null,
                    "keyFields": null,
                    "fields": {
                      "price": { "cacheControl": { "maxAge": 0, "swr": null, "scope": null } }
                    }
                  }
                }
              }
            "#,
        )
        .unwrap();

        let documents = split(&input, &schema, &manifest).unwrap();
        assert_eq!(documents.len(), 2);

        assert_eq!(
            documents.get(&Some(3600)).unwrap().print(),
            dedent(
                r#"
                  {
                    puma {
                      products {
                        name
                        id
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );

        assert_eq!(
            documents.get(&Some(0)).unwrap().print(),
            dedent(
                r#"
                  {
                    puma {
                      products {
                        price
                        id
                      }
                    }
                  }
                "#,
            )
            .trim(),
        );
    }
}

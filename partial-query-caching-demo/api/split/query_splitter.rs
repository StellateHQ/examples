use std::collections::HashMap;

use anyhow::anyhow;
use gql_query::{
    ast::{ASTContext, Field, InlineFragment, OperationDefinition, Selection, SelectionSet},
    schema::{Schema, SchemaFields, SchemaType},
    toolshed::list::List,
    visit::{Path, VisitFlow, VisitInfo, Visitor},
};

use crate::manifest::Manifest;

type SelectionSetSplitMap<'a> = HashMap<u64, SelectionSet<'a>>;
type SelectionsSplitMap<'a> = HashMap<u64, Vec<Selection<'a>>>;

#[derive(Debug)]
pub struct SelectionSplits<'a> {
    pub selection_set: SelectionSet<'a>,
    pub max_age: Option<u64>,
    pub splits: SelectionSetSplitMap<'a>,
    pub path: Path,
}

#[derive(Debug)]
pub struct QuerySplitter<'a> {
    pub result: Option<anyhow::Result<SelectionSplits<'a>>>,

    schema: &'a Schema<'a>,
    manifest: &'a Manifest,
    operation_name: &'a Option<String>,
    current_max_age: Option<u64>,
    type_stack: Vec<SchemaType<'a>>,
}

impl<'a> QuerySplitter<'a> {
    pub fn new(
        schema: &'a Schema<'a>,
        manifest: &'a Manifest,
        operation_name: &'a Option<String>,
    ) -> Self {
        Self {
            result: None,

            schema,
            manifest,
            operation_name,
            current_max_age: None,
            type_stack: vec![],
        }
    }
}

impl<'a> Visitor<'a, &'a ASTContext> for QuerySplitter<'a> {
    fn enter_operation(
        &mut self,
        _ctx: &&'a ASTContext,
        operation: &'a OperationDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if operation.name.map(|n| n.name) != self.operation_name.as_ref().map(|s| s.as_str()) {
            return VisitFlow::Skip;
        }

        match self
            .schema
            .get_root_type(operation.operation)
            .and_then(|obj| self.schema.get_type(obj.name))
        {
            Some(root_type) => {
                self.current_max_age = self.manifest.type_max_age(root_type.name());
                self.type_stack.push(root_type.clone());
                VisitFlow::Next
            }
            None => {
                self.result = Some(Err(anyhow!(
                    "Could not find root type for kind {:?}",
                    operation.operation
                )));
                VisitFlow::Break
            }
        }
    }

    fn enter_selection_set(
        &mut self,
        ctx: &&'a ASTContext,
        selection_set: &'a SelectionSet<'a>,
        info: &VisitInfo,
    ) -> VisitFlow {
        if selection_set.selections.is_empty() {
            return VisitFlow::Skip;
        }

        let current_type = self.type_stack.last();
        if current_type.is_none() {
            self.result = Some(Err(anyhow!(
                "Entered selection set without knowing its type"
            )));
            return VisitFlow::Break;
        }
        let current_type_name = current_type.unwrap().name();

        self.current_max_age = min(
            self.current_max_age,
            self.manifest.type_max_age(current_type_name),
        );

        let mut min_max_age = self.current_max_age;
        for selection in selection_set.selections {
            let field = match selection {
                Selection::Field(field) => field,
                _ => continue,
            };

            min_max_age = min(
                min_max_age,
                self.manifest.field_max_age(current_type_name, field.name),
            );
        }

        if min_max_age.is_none() {
            return VisitFlow::Next;
        }
        let min_max_age = min_max_age.unwrap();

        let mut key_fields = vec![];
        let mut non_splittable = vec![];
        let mut selections_split = SelectionsSplitMap::new();
        for selection in selection_set.selections {
            let field = match selection {
                Selection::Field(field) => field,
                _ => {
                    non_splittable.push(*selection);
                    continue;
                }
            };

            if self
                .manifest
                .key_field_names(current_type_name)
                .contains(&field.name)
            {
                key_fields.push(selection);
                continue;
            }

            if let Some(field_max_age) = self
                .manifest
                .field_max_age(current_type_name, field.name)
                .or(self.current_max_age)
            {
                if field_max_age > min_max_age {
                    if let Some(selections) = selections_split.get_mut(&field_max_age) {
                        selections.push(*selection);
                    } else {
                        selections_split.insert(field_max_age, vec![*selection]);
                    }
                } else {
                    non_splittable.push(*selection);
                }
            } else {
                non_splittable.push(*selection);
            }
        }

        self.current_max_age = Some(min_max_age);

        match (selections_split.len(), non_splittable.len()) {
            (0, _) | (1, 0) => VisitFlow::Next,
            _ => {
                let mut splits = SelectionSetSplitMap::default();
                for (key, mut selections) in selections_split {
                    for key_field in key_fields.iter() {
                        selections.push(**key_field);
                    }
                    splits.insert(
                        key,
                        SelectionSet {
                            selections: list_from_vec(&ctx, selections),
                        },
                    );
                }

                for key_field in key_fields {
                    non_splittable.push(*key_field);
                }

                self.result = Some(Ok(SelectionSplits {
                    selection_set: SelectionSet {
                        selections: list_from_vec(&ctx, non_splittable),
                    },
                    max_age: self.current_max_age,
                    splits,
                    path: info.path.clone(),
                }));
                VisitFlow::Break
            }
        }
    }

    fn enter_field(
        &mut self,
        _ctx: &&'a ASTContext,
        field: &'a Field<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if field.name.starts_with("__") {
            return VisitFlow::Skip;
        }

        let current_type = self.type_stack.last();
        let field_definition = match current_type {
            Some(SchemaType::Object(object)) => object.get_field(field.name),
            Some(SchemaType::Interface(interface)) => interface.get_field(field.name),
            current_type => {
                self.result = Some(Err(anyhow!(
                    "Tried to handle field {} inside type {:?} which is not a object type or interface type",
                    field.name,
                    current_type
                )));
                return VisitFlow::Break;
            }
        };
        if field_definition.is_none() {
            self.result = Some(Err(anyhow!(
                "Could not find field {} inside type {}",
                field.name,
                current_type.map(|t| t.name()).unwrap_or_default()
            )));
            return VisitFlow::Break;
        }
        let field_definition = field_definition.unwrap();

        let field_type = field_definition.output_type.of_type().into_schema_type();

        self.type_stack.push(field_type);

        VisitFlow::Next
    }

    fn leave_field(
        &mut self,
        _ctx: &&'a ASTContext,
        _field: &'a Field<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.type_stack.pop();
        VisitFlow::Next
    }

    fn enter_inline_fragment(
        &mut self,
        _ctx: &&'a ASTContext,
        inline_fragment: &'a InlineFragment<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if let Some(type_condition) = inline_fragment.type_condition {
            match self.schema.get_type(type_condition.name) {
                Some(current_type) => self.type_stack.push(*current_type),
                None => {
                    self.result = Some(Err(anyhow!(
                        "Entered inline fragment with unknown type {} in type condition",
                        type_condition.name
                    )));
                    return VisitFlow::Break;
                }
            }
        } else {
            match self.type_stack.last() {
                Some(current_type) => self.type_stack.push(*current_type),
                None => {
                    self.result = Some(Err(anyhow!(
                        "Entered inline fragment without type condition and without knowing its parent type"
                    )));
                    return VisitFlow::Break;
                }
            }
        }
        VisitFlow::Next
    }

    fn leave_inline_fragment(
        &mut self,
        _ctx: &&'a ASTContext,
        _inline_fragment: &'a InlineFragment<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.type_stack.pop();
        VisitFlow::Next
    }
}

fn list_from_vec<'a, T: Copy>(ctx: &'a ASTContext, vec: Vec<T>) -> List<'a, T> {
    let list = List::empty();
    for item in vec.into_iter().rev() {
        list.prepend(&ctx.arena, item);
    }
    list
}

fn min(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(a), Some(b)) => {
            if a <= b {
                Some(a)
            } else {
                Some(b)
            }
        }
        (a, b) => a.or(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{MANIFEST, SCHEMA};
    use gql_query::{
        ast::{Document, ParseNode, PrintNode},
        schema::{BuildClientSchema, IntrospectionQuery},
        visit::VisitNode,
    };
    use textwrap::dedent;

    #[test]
    fn top_level_fields() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            r#"
              query {
                lowMaxAge
                highMaxAge
                noMaxAge
              }
            "#,
        )
        .unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let mut splitter = QuerySplitter::new(schema, &manifest, &None);
        document.visit(&&ctx, &mut splitter);

        let result = splitter.result.unwrap().unwrap();

        assert_eq!(result.path.to_string(), "0.selectionSet");

        assert_eq!(
            result.selection_set.print(),
            dedent(
                r#"
                  {
                    lowMaxAge
                    noMaxAge
                  }
                "#
            )
            .trim()
        );
        assert_eq!(result.max_age, Some(100));

        assert_eq!(result.splits.len(), 1);
        assert_eq!(
            result.splits.get(&200).unwrap().print(),
            dedent(
                r#"
                  {
                    highMaxAge
                  }
                "#
            )
            .trim()
        );
    }

    #[test]
    fn nested_fields() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            r#"
              query {
                nested {
                  lowMaxAge
                  highMaxAge
                  noMaxAge
                }
              }
            "#,
        )
        .unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let mut splitter = QuerySplitter::new(schema, &manifest, &None);
        document.visit(&&ctx, &mut splitter);

        let result = splitter.result.unwrap().unwrap();

        assert_eq!(result.path.to_string(), "0.selectionSet.0.selectionSet");

        assert_eq!(
            result.selection_set.print(),
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
        assert_eq!(result.max_age, Some(100));

        assert_eq!(result.splits.len(), 1);
        assert_eq!(
            result.splits.get(&200).unwrap().print(),
            dedent(
                r#"
                  {
                    highMaxAge
                  }
                "#,
            )
            .trim(),
        );
    }

    #[test]
    fn combined() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            r#"
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
        )
        .unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let mut splitter = QuerySplitter::new(schema, &manifest, &None);
        document.visit(&&ctx, &mut splitter);

        let result = splitter.result.unwrap().unwrap();

        assert_eq!(result.path.to_string(), "0.selectionSet");

        assert_eq!(
            result.selection_set.print(),
            dedent(
                r#"
                {
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
            )
            .trim(),
        );
        assert_eq!(result.max_age, Some(0));

        assert_eq!(result.splits.len(), 2);
        assert_eq!(
            result.splits.get(&100).unwrap().print(),
            dedent(
                r#"
                {
                  lowMaxAge
                }
              "#,
            )
            .trim(),
        );
        assert_eq!(
            result.splits.get(&200).unwrap().print(),
            dedent(
                r#"
                {
                  highMaxAge
                }
              "#,
            )
            .trim(),
        );
    }

    #[test]
    fn inline_fragments() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            r#"
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
        )
        .unwrap();

        let introspection: IntrospectionQuery = serde_json::from_str(SCHEMA).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        let mut splitter = QuerySplitter::new(schema, &manifest, &None);
        document.visit(&&ctx, &mut splitter);

        let result = splitter.result.unwrap().unwrap();

        assert_eq!(
            result.path.to_string(),
            "0.selectionSet.0.selectionSet.1.selectionSet"
        );

        assert_eq!(
            result.selection_set.print(),
            dedent(
                r#"
                  {
                    text
                  }
                "#
            )
            .trim()
        );
        assert_eq!(result.max_age, Some(600));

        assert_eq!(result.splits.len(), 1);
        assert_eq!(
            result.splits.get(&900).unwrap().print(),
            dedent(
                r#"
                {
                  authors {
                    name
                  }
                }
              "#
            )
            .trim()
        );
    }
}

use gql_query::{
    ast::{ASTContext, Field, InlineFragment, OperationDefinition, Selection, SelectionSet},
    error::Error,
    visit::{self, Folder, Path, PathSegment, VisitInfo},
};
use std::collections::VecDeque;

pub struct SelectionSetExtractor<'a> {
    selection_indices: VecDeque<usize>,
    selection_set: SelectionSet<'a>,
}

impl<'a> SelectionSetExtractor<'a> {
    pub fn new(path: Path, selection_set: SelectionSet<'a>) -> Self {
        let selection_indices = path
            .segments
            .iter()
            .skip(1)
            .filter_map(|segment| match segment {
                PathSegment::Index(index) => Some(*index),
                _ => None,
            })
            .collect::<VecDeque<_>>();

        Self {
            selection_indices,
            selection_set,
        }
    }
}

impl<'a> Folder<'a> for SelectionSetExtractor<'a> {
    fn enter_operation(
        &mut self,
        _ctx: &'a ASTContext,
        operation: OperationDefinition<'a>,
        _info: &VisitInfo,
    ) -> visit::Result<OperationDefinition<'a>> {
        if self.selection_indices.len() == 0 {
            Ok(OperationDefinition {
                selection_set: self.selection_set,
                ..operation
            })
        } else {
            Ok(operation)
        }
    }

    fn selection_set(
        &mut self,
        ctx: &'a ASTContext,
        selection_set: SelectionSet<'a>,
        _info: &VisitInfo,
    ) -> visit::Result<SelectionSet<'a>> {
        if let Some(index) = self.selection_indices.pop_front() {
            let selection = selection_set
                .selections
                .iter()
                .nth(index)
                .ok_or(Error::new(
                    format!("No selection with index {index} in selection set"),
                    None,
                ))?;

            let selection_set = SelectionSet::default();
            if self.selection_indices.len() == 0 {
                match *selection {
                    Selection::Field(field) => {
                        selection_set.selections.prepend(
                            &ctx.arena,
                            Selection::Field(Field {
                                selection_set: self.selection_set,
                                ..field
                            }),
                        );
                    }
                    Selection::InlineFragment(inline_fragment) => {
                        selection_set.selections.prepend(
                            &ctx.arena,
                            Selection::InlineFragment(InlineFragment {
                                selection_set: self.selection_set,
                                ..inline_fragment
                            }),
                        );
                    }
                    _ => unreachable!(),
                }
            } else {
                selection_set.selections.prepend(&ctx.arena, *selection);
            }
            Ok(selection_set)
        } else {
            Ok(selection_set)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gql_query::{
        ast::{Document, ParseNode, PrintNode},
        visit::FoldNode,
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

        let selection_set = SelectionSet::parse(&ctx, "{ highMaxAge }").unwrap();
        let mut extractor =
            SelectionSetExtractor::new(Path::try_from("0.selectionSet").unwrap(), *selection_set);
        assert_eq!(
            document.fold(&ctx, &mut extractor).unwrap().print(),
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

        let selection_set = SelectionSet::parse(&ctx, "{ highMaxAge }").unwrap();
        let mut extractor = SelectionSetExtractor::new(
            Path::try_from("0.selectionSet.0.selectionSet").unwrap(),
            *selection_set,
        );
        assert_eq!(
            document.fold(&ctx, &mut extractor).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      highMaxAge
                    }
                  }
                "#
            )
            .trim()
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

        let selection_set = SelectionSet::parse(&ctx, "{ highMaxAge }").unwrap();
        let mut extractor = SelectionSetExtractor::new(
            Path::try_from("0.selectionSet.4.selectionSet").unwrap(),
            *selection_set,
        );
        assert_eq!(
            document.fold(&ctx, &mut extractor).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      highMaxAge
                    }
                  }
                "#,
            )
            .trim()
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

        let selection_set = SelectionSet::parse(&ctx, "{ authors { name } }").unwrap();
        let mut extractor = SelectionSetExtractor::new(
            Path::try_from("0.selectionSet.0.selectionSet.1.selectionSet").unwrap(),
            *selection_set,
        );
        // TODO: We want to also preserve the "id" field inside the "node" because it's a key-field.
        assert_eq!(
            document.fold(&ctx, &mut extractor).unwrap().print(),
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
            .trim()
        );
    }
}

use gql_query::{
    ast::{ASTContext, Document, FragmentDefinition, InlineFragment, Selection, SelectionSet},
    toolshed::map::Map,
    visit::{Error, FoldDocument, Folder, Result, VisitInfo},
};

pub trait InlineFragments<'a> {
    fn inline_fragments(
        &'a self,
        ctx: &'a ASTContext,
        operation_name: Option<&'a str>,
    ) -> Result<()>;
}

impl<'a> InlineFragments<'a> for Document<'a> {
    fn inline_fragments(
        &'a self,
        ctx: &'a ASTContext,
        operation_name: Option<&'a str>,
    ) -> Result<()> {
        let fragment_map = self.fragments(&ctx);

        let mut folder = Fold { fragment_map };
        let document = self.fold_operation(&ctx, operation_name, &mut folder)?;

        let mut definitions = vec![];
        while let Some(definition) = document.definitions.shift() {
            definitions.push(definition);
        }

        self.definitions.clear();
        for definition in definitions.into_iter().rev() {
            if definition.operation().is_some() {
                self.definitions.prepend(&ctx.arena, *definition);
            }
        }

        Ok(())
    }
}

type FragmentMap<'a> = Map<'a, &'a str, &'a FragmentDefinition<'a>>;

struct Fold<'a> {
    fragment_map: FragmentMap<'a>,
}

impl<'a> Folder<'a> for Fold<'a> {
    fn selection_set(
        &mut self,
        ctx: &'a ASTContext,
        selection_set: SelectionSet<'a>,
        _info: &VisitInfo,
    ) -> Result<SelectionSet<'a>> {
        let mut selections = vec![];
        while let Some(selection) = selection_set.selections.shift() {
            if let Some(fragment_spread) = selection.fragment_spread() {
                match self.fragment_map.get(fragment_spread.name.name) {
                    Some(fragment_definition) => {
                        selections.push(Selection::InlineFragment(InlineFragment {
                            type_condition: Some(fragment_definition.type_condition),
                            directives: fragment_spread.directives,
                            selection_set: fragment_definition.selection_set,
                        }))
                    }
                    None => {
                        return Err(Error::new(
                            format!("Fragment {} does not exist", fragment_spread.name.name),
                            None,
                        ))
                    }
                };
            } else {
                selections.push(*selection);
            }
        }

        for selection in selections.iter().rev() {
            selection_set.selections.prepend(&ctx.arena, *selection);
        }

        Ok(selection_set)
    }
}

#[cfg(test)]
mod tests {
    use gql_query::ast::{ParseNode, PrintNode};
    use textwrap::dedent;

    use super::*;

    #[test]
    fn operation_before_fragments() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            r#"
              {
                hello
                ...World
                ...Again @custom
              }

              fragment World on Query {
                 world
                 ...Again
              }

              fragment Again on Query {
                again
              }
            "#,
        )
        .unwrap();
        document.inline_fragments(&ctx, None).unwrap();
        assert_eq!(
            document.print(),
            dedent(
                r#"
                  {
                    hello
                    ... on Query {
                      world
                      ... on Query {
                        again
                      }
                    }
                    ... on Query @custom {
                      again
                    }
                  }
                "#
            )
            .trim()
        );
    }

    #[test]
    fn fragments_before_operation() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            r#"
              fragment Again on Query {
                again
              }

              fragment World on Query {
                 ...Again
                 world
              }

              {
                ...Again @custom
                ...World
                hello
              }
            "#,
        )
        .unwrap();
        document.inline_fragments(&ctx, None).unwrap();
        assert_eq!(
            document.print(),
            dedent(
                r#"
                  {
                    ... on Query @custom {
                      again
                    }
                    ... on Query {
                      ... on Query {
                        again
                      }
                      world
                    }
                    hello
                  }
                "#
            )
            .trim()
        );
    }
}

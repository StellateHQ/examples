use gql_query::{
    ast::{ASTContext, OperationDefinition, SelectionSet},
    visit::{self, Folder, Path, VisitInfo},
};

pub struct SelectionSetReplacer<'a> {
    path: Path,
    selection_set: SelectionSet<'a>,
}

impl<'a> SelectionSetReplacer<'a> {
    pub fn new(path: Path, selection_set: SelectionSet<'a>) -> Self {
        Self {
            path,
            selection_set,
        }
    }
}

impl<'a> Folder<'a> for SelectionSetReplacer<'a> {
    fn enter_operation(
        &mut self,
        _ctx: &'a ASTContext,
        operation: OperationDefinition<'a>,
        info: &VisitInfo,
    ) -> visit::Result<OperationDefinition<'a>> {
        if self.path == info.path {
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
        _ctx: &'a ASTContext,
        selection_set: SelectionSet<'a>,
        info: &VisitInfo,
    ) -> visit::Result<SelectionSet<'a>> {
        if self.path == info.path {
            Ok(self.selection_set)
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

        let selection_set = SelectionSet::parse(&ctx, "{ noMaxAge lowMaxAge }").unwrap();
        let mut replacer =
            SelectionSetReplacer::new(Path::try_from("0.selectionSet").unwrap(), *selection_set);
        assert_eq!(
            document.fold(&ctx, &mut replacer).unwrap().print(),
            dedent(
                r#"
                  {
                    noMaxAge
                    lowMaxAge
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

        let selection_set = SelectionSet::parse(&ctx, "{ noMaxAge lowMaxAge }").unwrap();
        let mut replacer = SelectionSetReplacer::new(
            Path::try_from("0.selectionSet.0.selectionSet").unwrap(),
            *selection_set,
        );
        assert_eq!(
            document.fold(&ctx, &mut replacer).unwrap().print(),
            dedent(
                r#"
                  {
                    nested {
                      noMaxAge
                      lowMaxAge
                    }
                  }
                "#
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

        let selection_set = SelectionSet::parse(&ctx, "{ text }").unwrap();
        let mut replacer = SelectionSetReplacer::new(
            Path::try_from("0.selectionSet.0.selectionSet.1.selectionSet").unwrap(),
            *selection_set,
        );
        assert_eq!(
            document.fold(&ctx, &mut replacer).unwrap().print(),
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
                "#
            )
            .trim()
        );
    }
}

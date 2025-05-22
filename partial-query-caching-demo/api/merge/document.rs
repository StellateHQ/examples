use gql_query::ast::{ASTContext, Definition, Document};

pub fn merge_documents<'a, 'b>(
    ctx: &'a ASTContext,
    document_a: &'a Document<'a>,
    document_b: &'a Document<'a>,
    operation_name: Option<&'b str>,
) -> Option<&'a Document<'a>> {
    match (
        document_a.operation(operation_name),
        document_b.operation(operation_name),
    ) {
        (Ok(operation_a), Ok(operation_b)) => {
            for selection in operation_a.selection_set {
                operation_b
                    .selection_set
                    .selections
                    .prepend(&ctx.arena, *selection);
            }

            for definition in document_a.definitions {
                match definition {
                    Definition::Operation(operation) if operation != operation_a => {
                        document_b
                            .definitions
                            .prepend(&ctx.arena, Definition::Operation(*operation));
                    }
                    Definition::Fragment(fragment) => {
                        document_b
                            .definitions
                            .prepend(&ctx.arena, Definition::Fragment(*fragment));
                    }
                    _ => {}
                };
            }

            Some(document_b)
        }
        (Ok(_), Err(_)) => Some(document_a),
        (Err(_), Ok(_)) => Some(document_b),
        (Err(_), Err(_)) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gql_query::ast::{ParseNode, PrintNode};
    use textwrap::dedent;

    #[test]
    fn merges_top_level_fields() {
        let ctx = ASTContext::new();

        let document_a = Document::parse(&ctx, "{ hello }").unwrap();
        let document_b = Document::parse(&ctx, "{ world }").unwrap();

        assert_eq!(
            merge_documents(&ctx, document_a, document_b, None)
                .unwrap()
                .print(),
            dedent(
                r#"
                  {
                    hello
                    world
                  }
                "#
            )
            .trim()
        );
    }

    #[test]
    fn merges_nested_fields() {
        let ctx = ASTContext::new();

        let document_a = Document::parse(&ctx, "{ hello { world } }").unwrap();
        let document_b = Document::parse(&ctx, "{ hello { people } }").unwrap();

        assert_eq!(
            merge_documents(&ctx, document_a, document_b, None)
                .unwrap()
                .print(),
            dedent(
                r#"
                  {
                    hello {
                      world
                    }
                    hello {
                      people
                    }
                  }
                "#
            )
            .trim()
        );
    }

    #[test]
    fn merges_with_fragments() {
        let ctx = ASTContext::new();

        let document_a = Document::parse(
            &ctx,
            "{ ...HelloFragment } fragment HelloFragment on Query { hello }",
        )
        .unwrap();
        let document_b = Document::parse(
            &ctx,
            "{ ...WorldFragment } fragment WorldFragment on Query { world }",
        )
        .unwrap();

        assert_eq!(
            merge_documents(&ctx, document_a, document_b, None)
                .unwrap()
                .print(),
            dedent(
                r#"
                  fragment HelloFragment on Query {
                    hello
                  }

                  {
                    ...HelloFragment
                    ...WorldFragment
                  }

                  fragment WorldFragment on Query {
                    world
                  }
                "#
            )
            .trim()
        );
    }
}

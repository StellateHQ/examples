use super::super::{ValidationContext, ValidationRule};
use crate::{ast::*, visit::*};
use toolshed::set::Set;

/// Validates that no fragments the document defines have duplicate names.
/// Note: Operations and Fragments are allowed to share names.
///
/// See [`ValidationRule`]
/// [Reference](https://spec.graphql.org/October2021/#sec-Fragment-Name-Uniqueness)
#[derive(Default)]
pub struct UniqueFragmentNames<'a> {
    used_fragment_names: Set<'a, &'a str>,
}

impl<'a> ValidationRule<'a> for UniqueFragmentNames<'a> {}

impl<'a> Visitor<'a, ValidationContext<'a>> for UniqueFragmentNames<'a> {
    fn enter_fragment(
        &mut self,
        ctx: &ValidationContext<'a>,
        fragment: &'a FragmentDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if self.used_fragment_names.contains(fragment.name.name) {
            ctx.add_error("All defined fragments must have unique names");
            VisitFlow::Break
        } else {
            self.used_fragment_names
                .insert(ctx.arena, fragment.name.name);
            VisitFlow::Skip
        }
    }

    fn enter_operation(
        &mut self,
        _ctx: &ValidationContext,
        _operation: &'a OperationDefinition,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_fragment_names() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "fragment Root on Query { __typename }").unwrap();
        UniqueFragmentNames::validate(&ctx, document).unwrap();
    }

    #[test]
    fn overlapping_fragment_names() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            "fragment Root on Query { __typename } fragment Root on Item { __typename }",
        )
        .unwrap();
        UniqueFragmentNames::validate(&ctx, document).unwrap_err();
    }
}

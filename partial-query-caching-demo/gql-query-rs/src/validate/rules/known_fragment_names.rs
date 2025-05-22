use super::super::{ValidationContext, ValidationRule};
use crate::{ast::*, visit::*};
use toolshed::set::Set;

/// Validate a document for all fragment names in spreads to be defined in the same document.
///
/// See [`ValidationRule`]
/// [Reference](https://spec.graphql.org/October2021/#sec-Fragment-spread-target-defined)
#[derive(Default)]
pub struct KnownFragmentNames<'a> {
    fragment_names: Set<'a, &'a str>,
    fragment_spreads: Set<'a, &'a str>,
}

impl<'a> ValidationRule<'a> for KnownFragmentNames<'a> {}

impl<'a> Visitor<'a, ValidationContext<'a>> for KnownFragmentNames<'a> {
    fn enter_fragment(
        &mut self,
        ctx: &ValidationContext<'a>,
        fragment: &'a FragmentDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.fragment_names.insert(ctx.arena, fragment.name.name);
        VisitFlow::Next
    }

    fn enter_fragment_spread(
        &mut self,
        ctx: &ValidationContext<'a>,
        spread: &'a FragmentSpread<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.fragment_spreads.insert(ctx.arena, spread.name.name);
        VisitFlow::Skip
    }

    fn leave_document(
        &mut self,
        ctx: &ValidationContext<'a>,
        _document: &'a Document<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        for name in self.fragment_spreads {
            if !self.fragment_names.contains(name) {
                ctx.add_error("Only known fragments may occur in fragment spreads.");
            }
        }
        VisitFlow::Next
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _var_def: &'a VariableDefinition,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }

    fn enter_argument(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _argument: &'a Argument,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }

    fn enter_directive(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _directive: &'a Directive,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_spread() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            "query { ...Root } fragment Root on Query { __typename }",
        )
        .unwrap();
        KnownFragmentNames::validate(&ctx, document).unwrap();
    }

    #[test]
    fn invalid_spread() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "query { ...Unknown }").unwrap();
        KnownFragmentNames::validate(&ctx, document).unwrap_err();
    }
}

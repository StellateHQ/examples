use super::super::{ValidationContext, ValidationRule};
use crate::{ast::*, visit::*};
use toolshed::set::Set;

/// Validates that no operation the document defines has duplicate variable names in its variable
/// definitions.
///
/// See [`ValidationRule`]
/// [Reference](https://spec.graphql.org/October2021/#sec-Variable-Uniqueness)
#[derive(Default)]
pub struct UniqueVariableNames<'a> {
    used_variable_names: Set<'a, &'a str>,
}

impl<'a> ValidationRule<'a> for UniqueVariableNames<'a> {}

impl<'a> Visitor<'a, ValidationContext<'a>> for UniqueVariableNames<'a> {
    fn enter_operation(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _operation: &'a OperationDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.used_variable_names.clear();
        VisitFlow::Next
    }

    fn enter_variable_definition(
        &mut self,
        ctx: &ValidationContext<'a>,
        var_def: &'a VariableDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if self.used_variable_names.contains(var_def.variable.name) {
            ctx.add_error("All defined variables per operation must have unique names");
            VisitFlow::Break
        } else {
            self.used_variable_names
                .insert(ctx.arena, var_def.variable.name);
            VisitFlow::Skip
        }
    }

    fn enter_selection_set(
        &mut self,
        _ctx: &ValidationContext,
        _selection_set: &'a SelectionSet,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }

    fn enter_directive(
        &mut self,
        _ctx: &ValidationContext,
        _fragment: &'a Directive,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }

    fn enter_fragment(
        &mut self,
        _ctx: &ValidationContext,
        _fragment: &'a FragmentDefinition,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_variables() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "query ($a: Int, $b: Int) { __typename }").unwrap();
        UniqueVariableNames::validate(&ctx, document).unwrap();
    }

    #[test]
    fn overlapping_variables() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "query ($a: Int, $a: Int) { __typename }").unwrap();
        UniqueVariableNames::validate(&ctx, document).unwrap_err();
    }
}

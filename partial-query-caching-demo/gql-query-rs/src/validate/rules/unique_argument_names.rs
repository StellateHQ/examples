use super::super::{ValidationContext, ValidationRule};
use crate::{ast::*, visit::*};
use toolshed::set::Set;

/// Validates that no arguments anywhere contain duplicate names.
///
/// See [`ValidationRule`]
/// [Reference](https://spec.graphql.org/October2021/#sec-Argument-Uniqueness)
#[derive(Default)]
pub struct UniqueArgumentNames<'a> {
    used_argument_names: Set<'a, &'a str>,
}

impl<'a> ValidationRule<'a> for UniqueArgumentNames<'a> {}

impl<'a> Visitor<'a, ValidationContext<'a>> for UniqueArgumentNames<'a> {
    fn enter_field(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _field: &'a Field<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.used_argument_names.clear();
        VisitFlow::Next
    }

    fn enter_directive(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _directive: &'a Directive<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.used_argument_names.clear();
        VisitFlow::Next
    }

    fn enter_argument(
        &mut self,
        ctx: &ValidationContext<'a>,
        argument: &'a Argument<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if self.used_argument_names.contains(argument.name) {
            ctx.add_error("All passed arguments per field must only be passed once");
            VisitFlow::Break
        } else {
            self.used_argument_names.insert(ctx.arena, argument.name);
            VisitFlow::Skip
        }
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _var_def: &'a VariableDefinition,
        _info: &VisitInfo,
    ) -> VisitFlow {
        VisitFlow::Skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_args() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "{ test(field: 1) }").unwrap();
        UniqueArgumentNames::validate(&ctx, document).unwrap();
    }

    #[test]
    fn overlapping_args() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "{ test(field: 1, field: 2) }").unwrap();
        UniqueArgumentNames::validate(&ctx, document).unwrap_err();
    }
}

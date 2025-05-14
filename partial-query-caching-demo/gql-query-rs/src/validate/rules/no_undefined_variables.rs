use super::super::{ValidationContext, ValidationRule};
use crate::{ast::*, visit::*};
use toolshed::{list::List, map::Map, set::Set, Arena};

#[derive(Default, Clone, Copy)]
struct OperationEdge<'a> {
    defined_vars: Set<'a, &'a str>,
    used_fragments: Set<'a, &'a str>,
}

#[derive(Default, Clone, Copy)]
struct FragmentEdge<'a> {
    used_vars: Set<'a, &'a str>,
    used_fragments: Set<'a, &'a str>,
}

/// Validate that a document defines all the variables it uses per operation
///
/// See [`ValidationRule`]
/// [Reference](https://spec.graphql.org/October2021/#sec-All-Variable-Uses-Defined)
pub struct NoUndefinedVariables<'a> {
    used_vars: Set<'a, &'a str>,
    defined_vars: Set<'a, &'a str>,
    used_fragments: Set<'a, &'a str>,
    operation_edges: List<'a, OperationEdge<'a>>,
    fragment_edges: Map<'a, &'a str, FragmentEdge<'a>>,
}

impl<'a> Default for NoUndefinedVariables<'a> {
    fn default() -> Self {
        NoUndefinedVariables {
            used_vars: Set::default(),
            defined_vars: Set::default(),
            used_fragments: Set::default(),
            operation_edges: List::empty(),
            fragment_edges: Map::default(),
        }
    }
}

impl<'a> ValidationRule<'a> for NoUndefinedVariables<'a> {}

impl<'a> Visitor<'a, ValidationContext<'a>> for NoUndefinedVariables<'a> {
    fn enter_variable_definition(
        &mut self,
        ctx: &ValidationContext<'a>,
        var_def: &'a VariableDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.defined_vars.insert(ctx.arena, var_def.variable.name);
        VisitFlow::Skip
    }

    fn enter_argument(
        &mut self,
        ctx: &ValidationContext<'a>,
        argument: &'a Argument,
        _info: &VisitInfo,
    ) -> VisitFlow {
        if let Value::Variable(var) = argument.value {
            self.used_vars.insert(ctx.arena, var.name);
        }
        VisitFlow::Skip
    }

    fn leave_operation(
        &mut self,
        ctx: &ValidationContext<'a>,
        _operation: &'a OperationDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        for var in self.used_vars {
            if !self.defined_vars.contains(var) {
                ctx.add_error(
                    "All variables used within operations must be defined on the operation",
                );
                return VisitFlow::Break;
            }
        }
        self.operation_edges.prepend(
            ctx.arena,
            OperationEdge {
                defined_vars: self.defined_vars,
                used_fragments: self.used_fragments,
            },
        );
        self.used_fragments.clear();
        self.used_vars.clear();
        self.defined_vars.clear();
        VisitFlow::Next
    }

    fn leave_fragment(
        &mut self,
        ctx: &ValidationContext<'a>,
        fragment: &'a FragmentDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        let name = fragment.name.name;
        self.fragment_edges.insert(
            ctx.arena,
            name,
            FragmentEdge {
                used_vars: self.used_vars,
                used_fragments: self.used_fragments,
            },
        );
        self.used_fragments.clear();
        self.used_vars.clear();
        VisitFlow::Next
    }

    fn enter_fragment_spread(
        &mut self,
        ctx: &ValidationContext<'a>,
        spread: &'a FragmentSpread<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        self.used_fragments.insert(ctx.arena, spread.name.name);
        VisitFlow::Skip
    }

    fn leave_document(
        &mut self,
        ctx: &ValidationContext<'a>,
        _document: &'a Document<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        let visited: Set<'a, &'a str> = Set::default();
        for operation_edge in self.operation_edges {
            if references_undefined_var(
                ctx.arena,
                &visited,
                &self.fragment_edges,
                &operation_edge.defined_vars,
                operation_edge.used_fragments,
            ) {
                ctx.add_error("All variables within fragments must be defined on the operation they're used in");
                return VisitFlow::Break;
            }
            visited.clear();
        }
        VisitFlow::Next
    }
}

fn references_undefined_var<'a>(
    arena: &'a Arena,
    visited: &Set<'a, &'a str>,
    fragment_edges: &Map<'a, &'a str, FragmentEdge<'a>>,
    defined_vars: &Set<'a, &'a str>,
    used_fragments: Set<'a, &'a str>,
) -> bool {
    for fragment_name in used_fragments {
        if !visited.contains(fragment_name) {
            visited.insert(arena, fragment_name);
            if let Some(edge) = fragment_edges.get(fragment_name) {
                for var in edge.used_vars {
                    if !defined_vars.contains(var) {
                        return true;
                    }
                }
                if references_undefined_var(
                    arena,
                    visited,
                    fragment_edges,
                    defined_vars,
                    edge.used_fragments,
                ) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defined_vars() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "query($var: Int) { field(x: $var), ...Frag } fragment Frag on Query { field(x: $var) } ").unwrap();
        NoUndefinedVariables::validate(&ctx, document).unwrap();
    }

    #[test]
    fn undefined_vars_on_operation() {
        let ctx = ASTContext::new();
        let document = Document::parse(&ctx, "query { field(x: $var) }").unwrap();
        NoUndefinedVariables::validate(&ctx, document).unwrap_err();
    }

    #[test]
    fn undefined_vars_on_fragments() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            "query { ...Frag } fragment Frag on Query { field(x: $var) } ",
        )
        .unwrap();
        NoUndefinedVariables::validate(&ctx, document).unwrap_err();
        let document = Document::parse(
            &ctx,
            "query { ...A } fragment A on A { ...B } fragment B on B { field(x: $var) } ",
        )
        .unwrap();
        NoUndefinedVariables::validate(&ctx, document).unwrap_err();
    }
}

use super::super::{ValidationContext, ValidationRule};
use crate::{ast::*, visit::*};
use toolshed::{map::Map, set::Set, Arena};

/// Validate that a document does not contain fragments that are spread within themselves, creating a loop.
///
/// See [`ValidationRule`]
/// [Reference](https://spec.graphql.org/October2021/#sec-Fragment-spreads-must-not-form-cycles)
#[derive(Default)]
pub struct NoFragmentCycles<'a> {
    fragment_edges: Map<'a, &'a str, Set<'a, &'a str>>,
    used_fragments: Set<'a, &'a str>,
}

impl<'a> ValidationRule<'a> for NoFragmentCycles<'a> {}

impl<'a> Visitor<'a, ValidationContext<'a>> for NoFragmentCycles<'a> {
    fn enter_operation(
        &mut self,
        _ctx: &ValidationContext<'a>,
        _operation: &'a OperationDefinition<'a>,
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
        self.used_fragments.clear();
        VisitFlow::Next
    }

    fn leave_fragment(
        &mut self,
        ctx: &ValidationContext<'a>,
        fragment: &'a FragmentDefinition<'a>,
        _info: &VisitInfo,
    ) -> VisitFlow {
        let name = fragment.name.name;
        self.fragment_edges
            .insert(ctx.arena, name, self.used_fragments);
        self.used_fragments.clear();
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
        for (name, _) in self.fragment_edges {
            if contains_edge(ctx.arena, &visited, name, name, &self.fragment_edges) {
                ctx.add_error("Cannot spread fragments within themselves");
                return VisitFlow::Break;
            }
            visited.clear();
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

fn contains_edge<'a>(
    arena: &'a Arena,
    visited: &Set<'a, &'a str>,
    toplevel_name: &'a str,
    current_name: &'a str,
    fragment_edges: &Map<'a, &'a str, Set<'a, &'a str>>,
) -> bool {
    if visited.contains(current_name) {
        true
    } else if let Some(edges) = fragment_edges.get(current_name) {
        visited.insert(arena, current_name);
        if edges.contains(toplevel_name) {
            true
        } else {
            for next_name in edges {
                if contains_edge(arena, visited, toplevel_name, next_name, fragment_edges) {
                    return true;
                }
            }
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_fragment_spreads() {
        let ctx = ASTContext::new();
        let document = Document::parse(
            &ctx,
            "fragment A on A { ...B } fragment B on B { __typename }",
        )
        .unwrap();
        NoFragmentCycles::validate(&ctx, document).unwrap();
    }

    #[test]
    fn cycling_fragment_spreads() {
        let ctx = ASTContext::new();
        let document =
            Document::parse(&ctx, "fragment A on A { ...B } fragment B on B { ...A }").unwrap();
        NoFragmentCycles::validate(&ctx, document).unwrap_err();
        let document = Document::parse(
            &ctx,
            "fragment A on A { ...B } fragment B on B { ...C } fragment C on C { ...A }",
        )
        .unwrap();
        NoFragmentCycles::validate(&ctx, document).unwrap_err();
        let document = Document::parse(&ctx, "fragment D on D { ...C } fragment A on A { ...B } fragment B on B { ...C } fragment C on C { ...A }").unwrap();
        NoFragmentCycles::validate(&ctx, document).unwrap_err();
        let document = Document::parse(&ctx, "fragment D on D { ...E } fragment A on A { ...B } fragment B on B { ...C } fragment C on C { ...A } fragment E on E { __typename }").unwrap();
        NoFragmentCycles::validate(&ctx, document).unwrap_err();
    }
}

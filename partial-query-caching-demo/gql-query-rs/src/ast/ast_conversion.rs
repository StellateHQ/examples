#![allow(clippy::needless_update)]

use super::ast::*;
use crate::error::Result;
use toolshed::list::{List, ListBuilder, ListIter};

macro_rules! default_iter {
    (impl $imp:tt for $t:tt.$p:ident) => {
        impl<'a> $imp for $t<'a> {
            #[inline]
            fn default() -> Self {
                Self { $p: List::empty() }
            }
        }
    };
}

macro_rules! into_iter {
    (impl $imp:tt<$it:ident> for $t:tt.$p:ident) => {
        impl<'a> $imp for $t<'a> {
            type Item = &'a $it<'a>;
            type IntoIter = ListIter<'a, $it<'a>>;
            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                self.$p.into_iter()
            }
        }

        impl<'a> $imp for &'a $t<'a> {
            type Item = &'a $it<'a>;
            type IntoIter = ListIter<'a, $it<'a>>;
            #[inline]
            fn into_iter(self) -> Self::IntoIter {
                self.$p.into_iter()
            }
        }
    };
}

macro_rules! from_iter_in {
    (impl $imp:tt<$it:ident> for $t:tt.$p:ident) => {
        impl<'a> $imp<'a, &'a $it<'a>> for $t<'a> {
            #[inline]
            fn from_iter_in<I>(iter: I, ctx: &'a ASTContext) -> Self
            where
                I: IntoIterator<Item = &'a $it<'a>>,
            {
                let mut iter = iter.into_iter();
                let $p = if let Some(item) = iter.next() {
                    let builder = ListBuilder::new(&ctx.arena, *item);
                    for item in iter {
                        builder.push(&ctx.arena, *item);
                    }
                    builder.as_list()
                } else {
                    List::empty()
                };
                $t {
                    $p,
                    ..$t::default()
                }
            }
        }

        impl<'a> $imp<'a, $it<'a>> for $t<'a> {
            #[inline]
            fn from_iter_in<I>(iter: I, ctx: &'a ASTContext) -> Self
            where
                I: IntoIterator<Item = $it<'a>>,
            {
                let mut iter = iter.into_iter();
                let $p = if let Some(item) = iter.next() {
                    let builder = ListBuilder::new(&ctx.arena, item);
                    for item in iter {
                        builder.push(&ctx.arena, item);
                    }
                    builder.as_list()
                } else {
                    List::empty()
                };
                $t {
                    $p,
                    ..$t::default()
                }
            }
        }

        impl<'a> $imp<'a, Result<$it<'a>>> for Result<$t<'a>> {
            #[inline]
            fn from_iter_in<I>(iter: I, ctx: &'a ASTContext) -> Self
            where
                I: IntoIterator<Item = Result<$it<'a>>>,
            {
                let mut iter = iter.into_iter();
                let $p = if let Some(item) = iter.next() {
                    let builder = ListBuilder::new(&ctx.arena, item?);
                    for item in iter {
                        builder.push(&ctx.arena, item?);
                    }
                    builder.as_list()
                } else {
                    List::empty()
                };
                Ok($t {
                    $p,
                    ..$t::default()
                })
            }
        }
    };
}

/// Converts an iterator of items into a given AST node.
///
/// This accepts an AST Context into which the
/// new node's list will be allocated into, hence assuming the context's lifetime.
/// This is implemented for all AST nodes with lists and its items, references of its items, and
/// iterators returning results of items.
/// See: [Result]
pub trait FromIteratorIn<'a, T> {
    fn from_iter_in<I>(iter: I, ctx: &'a ASTContext) -> Self
    where
        I: IntoIterator<Item = T>;
}

/// Collects an iterator of items into a given AST node list.
///
/// This accepts an AST Context into which the
/// new node's list will be allocated into, hence assuming the context's lifetime.
/// This is implemented for all AST nodes with lists and its items, references of its items, and
/// iterators returning results of items.
/// See: [Result]
pub trait CollectIn<'a>: Iterator + Sized {
    fn collect_in<C: FromIteratorIn<'a, Self::Item>>(self, ctx: &'a ASTContext) -> C {
        C::from_iter_in(self, ctx)
    }
}

impl<I: Iterator> CollectIn<'_> for I {}

default_iter! { impl Default for ListValue.children }
into_iter! { impl IntoIterator<Value> for ListValue.children }
from_iter_in! { impl FromIteratorIn<Value> for ListValue.children }

default_iter! { impl Default for ObjectValue.children }
into_iter! { impl IntoIterator<ObjectField> for ObjectValue.children }
from_iter_in! { impl FromIteratorIn<ObjectField> for ObjectValue.children }

default_iter! { impl Default for Arguments.children }
into_iter! { impl IntoIterator<Argument> for Arguments.children }
from_iter_in! { impl FromIteratorIn<Argument> for Arguments.children }

default_iter! { impl Default for Directives.children }
into_iter! { impl IntoIterator<Directive> for Directives.children }
from_iter_in! { impl FromIteratorIn<Directive> for Directives.children }

default_iter! { impl Default for SelectionSet.selections }
into_iter! { impl IntoIterator<Selection> for SelectionSet.selections }
from_iter_in! { impl FromIteratorIn<Selection> for SelectionSet.selections }

default_iter! { impl Default for VariableDefinitions.children }
into_iter! { impl IntoIterator<VariableDefinition> for VariableDefinitions.children }
from_iter_in! { impl FromIteratorIn<VariableDefinition> for VariableDefinitions.children }

/// Assumed minimum length of most short GraphQL queries
const MIN_CAPACITY: usize = 2048;

impl<'a> Default for Document<'a> {
    fn default() -> Self {
        Document {
            definitions: toolshed::list::List::empty(),
            size_hint: MIN_CAPACITY,
        }
    }
}

into_iter! { impl IntoIterator<Definition> for Document.definitions }
from_iter_in! { impl FromIteratorIn<Definition> for Document.definitions }

impl<'a> From<&'a str> for NamedType<'a> {
    #[inline]
    fn from(name: &'a str) -> Self {
        NamedType { name }
    }
}

impl<'a> From<&'a str> for Variable<'a> {
    #[inline]
    fn from(name: &'a str) -> Variable<'a> {
        Variable { name }
    }
}

impl<'a> From<bool> for BooleanValue {
    #[inline]
    fn from(value: bool) -> Self {
        BooleanValue { value }
    }
}

impl<'a> From<&'a str> for StringValue<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        StringValue { value }
    }
}

impl<'a> From<Variable<'a>> for Value<'a> {
    #[inline]
    fn from(x: Variable<'a>) -> Self {
        Value::Variable(x)
    }
}

impl<'a> From<StringValue<'a>> for Value<'a> {
    #[inline]
    fn from(x: StringValue<'a>) -> Self {
        Value::String(x)
    }
}

impl<'a> From<FloatValue<'a>> for Value<'a> {
    #[inline]
    fn from(x: FloatValue<'a>) -> Self {
        Value::Float(x)
    }
}

impl<'a> From<IntValue<'a>> for Value<'a> {
    #[inline]
    fn from(x: IntValue<'a>) -> Self {
        Value::Int(x)
    }
}

impl<'a> From<BooleanValue> for Value<'a> {
    #[inline]
    fn from(x: BooleanValue) -> Self {
        Value::Boolean(x)
    }
}

impl<'a> From<EnumValue<'a>> for Value<'a> {
    #[inline]
    fn from(x: EnumValue<'a>) -> Self {
        Value::Enum(x)
    }
}

impl<'a> From<ListValue<'a>> for Value<'a> {
    #[inline]
    fn from(x: ListValue<'a>) -> Self {
        Value::List(x)
    }
}

impl<'a> From<ObjectValue<'a>> for Value<'a> {
    #[inline]
    fn from(x: ObjectValue<'a>) -> Self {
        Value::Object(x)
    }
}

impl<'a> From<NamedType<'a>> for Type<'a> {
    #[inline]
    fn from(x: NamedType<'a>) -> Self {
        Type::NamedType(x)
    }
}

impl<'a> From<Field<'a>> for Selection<'a> {
    #[inline]
    fn from(x: Field<'a>) -> Self {
        Selection::Field(x)
    }
}

impl<'a> From<FragmentSpread<'a>> for Selection<'a> {
    #[inline]
    fn from(x: FragmentSpread<'a>) -> Self {
        Selection::FragmentSpread(x)
    }
}

impl<'a> From<InlineFragment<'a>> for Selection<'a> {
    #[inline]
    fn from(x: InlineFragment<'a>) -> Self {
        Selection::InlineFragment(x)
    }
}

impl<'a> From<OperationDefinition<'a>> for Definition<'a> {
    #[inline]
    fn from(x: OperationDefinition<'a>) -> Self {
        Definition::Operation(x)
    }
}

impl<'a> From<FragmentDefinition<'a>> for Definition<'a> {
    #[inline]
    fn from(x: FragmentDefinition<'a>) -> Self {
        Definition::Fragment(x)
    }
}

#[cfg(test)]
mod tests {
    use super::super::ast::*;

    #[test]
    fn list_value() {
        let ctx = ASTContext::new();
        let node = ListValue::from_iter_in(&[Value::Null], &ctx);
        assert_eq!(node, ListValue::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }

    #[test]
    fn object_value() {
        let ctx = ASTContext::new();
        let node = ObjectValue::from_iter_in(
            ctx.alloc([ObjectField {
                name: "field",
                value: Value::Null,
            }]),
            &ctx,
        );

        assert_eq!(node, ObjectValue::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }

    #[test]
    fn arguments() {
        let ctx = ASTContext::new();
        let node = Arguments::from_iter_in(
            ctx.alloc([Argument {
                name: "arg",
                value: Value::Null,
            }]),
            &ctx,
        );

        assert_eq!(node, Arguments::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }

    #[test]
    fn directives() {
        let ctx = ASTContext::new();
        let node = Directives::from_iter_in(
            ctx.alloc([Directive {
                name: "arg",
                arguments: Arguments::default(),
            }]),
            &ctx,
        );

        assert_eq!(node, Directives::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }

    #[test]
    fn selection_set() {
        let ctx = ASTContext::new();
        let node = SelectionSet::from_iter_in(
            ctx.alloc([Selection::FragmentSpread(FragmentSpread {
                name: NamedType { name: "Frag" },
                directives: Directives::default(),
            })]),
            &ctx,
        );

        assert_eq!(node, SelectionSet::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }

    #[test]
    fn var_definitions() {
        let ctx = ASTContext::new();
        let node = VariableDefinitions::from_iter_in(
            ctx.alloc([VariableDefinition {
                variable: Variable { name: "var" },
                of_type: Type::NamedType(NamedType { name: "String" }),
                default_value: Value::Null,
                directives: Directives::default(),
            }]),
            &ctx,
        );

        assert_eq!(node, VariableDefinitions::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }

    #[test]
    fn document() {
        let ctx = ASTContext::new();
        let node = Document::from_iter_in(
            ctx.alloc([Definition::Fragment(FragmentDefinition {
                name: NamedType { name: "Frag" },
                type_condition: NamedType { name: "Query" },
                directives: Directives::default(),
                selection_set: SelectionSet::default(),
            })]),
            &ctx,
        );

        assert_eq!(node, Document::from_iter_in(node, &ctx));
        assert_eq!(node, node.into_iter().collect_in(&ctx));
    }
}

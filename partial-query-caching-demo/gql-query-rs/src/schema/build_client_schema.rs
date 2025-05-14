use super::introspection::{IntrospectionQuery, IntrospectionSchema};
use super::schema::Schema;
use crate::ast::ASTContext;

pub(crate) mod private {
    use super::super::{introspection::*, schema::*};
    use super::ASTContext;
    use toolshed::map::Map;

    #[derive(Clone, Copy)]
    pub struct BuildSchemaContext<'a> {
        pub(crate) ctx: &'a ASTContext,
        pub(crate) types: Map<'a, &'a str, &'a SchemaType<'a>>,
    }

    impl<'a> BuildSchemaContext<'a> {
        pub(crate) fn new(ctx: &'a ASTContext) -> Self {
            BuildSchemaContext {
                types: Map::default(),
                ctx,
            }
        }

        #[inline]
        pub fn add_type(&'a self, named_type: SchemaType<'a>) {
            let name = self.ctx.alloc_str(named_type.name());
            self.types
                .insert(&self.ctx.arena, name, self.ctx.alloc(named_type));
        }

        #[inline]
        pub fn get_type(&'a self, name: &str) -> Option<&'a SchemaType<'a>> {
            let name = self.ctx.alloc_str(name);
            self.types.get(name)
        }

        pub fn build_schema<'data>(
            &'a self,
            introspection: &'data IntrospectionSchema<'data>,
        ) -> &'a Schema<'a> {
            introspection.types.iter().for_each(|introspection_type| {
                BuildSchemaType::on_create(introspection_type, self);
            });

            self.types.iter().zip(introspection.types.iter()).for_each(
                |(schema_type, introspection_type)| {
                    BuildSchemaType::on_build(introspection_type, self, &schema_type.1);
                },
            );

            let query_type = introspection
                .query_type
                .as_ref()
                .and_then(|type_ref| self.get_type(&type_ref.name))
                .and_then(|schema_type| schema_type.object());
            let mutation_type = introspection
                .mutation_type
                .as_ref()
                .and_then(|type_ref| self.get_type(&type_ref.name))
                .and_then(|schema_type| schema_type.object());
            let subscription_type = introspection
                .subscription_type
                .as_ref()
                .and_then(|type_ref| self.get_type(&type_ref.name))
                .and_then(|schema_type| schema_type.object());
            self.ctx.alloc(Schema {
                query_type,
                mutation_type,
                subscription_type,
                types: self.types,
            })
        }
    }

    pub trait BuildSchemaType<'arena, 'data, T: Copy>: Sized {
        fn on_create(&'data self, ctx: &'arena BuildSchemaContext<'arena>) -> &'arena T;
        fn on_build(&'data self, _ctx: &'arena BuildSchemaContext<'arena>, _named_type: &'arena T) {
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaType<'arena>>
        for IntrospectionType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaType<'arena> {
            ctx.ctx.alloc(match self {
                IntrospectionType::Scalar(scalar) => SchemaType::Scalar(scalar.on_create(ctx)),
                IntrospectionType::Object(object) => SchemaType::Object(object.on_create(ctx)),
                IntrospectionType::Interface(interface) => {
                    SchemaType::Interface(interface.on_create(ctx))
                }
                IntrospectionType::Union(union_type) => {
                    SchemaType::Union(union_type.on_create(ctx))
                }
                IntrospectionType::Enum(enum_type) => SchemaType::Enum(enum_type.on_create(ctx)),
                IntrospectionType::InputObject(input_object) => {
                    SchemaType::InputObject(input_object.on_create(ctx))
                }
            })
        }

        #[inline]
        fn on_build(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
            named_type: &'arena SchemaType<'arena>,
        ) {
            match (self, named_type) {
                (IntrospectionType::Scalar(scalar), SchemaType::Scalar(schema_scalar)) => {
                    scalar.on_build(ctx, schema_scalar)
                }
                (IntrospectionType::Object(object), SchemaType::Object(schema_object)) => {
                    object.on_build(ctx, schema_object)
                }
                (
                    IntrospectionType::Interface(interface),
                    SchemaType::Interface(schema_interface),
                ) => interface.on_build(ctx, schema_interface),
                (IntrospectionType::Union(union_type), SchemaType::Union(schema_union)) => {
                    union_type.on_build(ctx, schema_union)
                }
                (IntrospectionType::Enum(enum_type), SchemaType::Enum(schema_enum)) => {
                    enum_type.on_build(ctx, schema_enum)
                }
                (IntrospectionType::InputObject(input), SchemaType::InputObject(schema_input)) => {
                    input.on_build(ctx, schema_input)
                }
                _ => {}
            };
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaScalar<'arena>>
        for IntrospectionScalarType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaScalar<'arena> {
            let scalar = ctx
                .ctx
                .alloc(SchemaScalar::new(ctx.ctx.alloc_str(self.name)));
            ctx.add_type(SchemaType::Scalar(scalar));
            scalar
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaEnum<'arena>>
        for IntrospectionEnumType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaEnum<'arena> {
            let name = ctx.ctx.alloc_str(self.name);
            let enum_type = ctx.ctx.alloc(SchemaEnum::new(name));
            for value in self.enum_values.iter() {
                let value_name = ctx.ctx.alloc_str(value.name);
                enum_type.add_value(ctx.ctx, value_name);
            }
            ctx.add_type(SchemaType::Enum(enum_type));
            enum_type
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaUnion<'arena>>
        for IntrospectionUnionType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaUnion<'arena> {
            let name = ctx.ctx.alloc_str(self.name);
            let union = ctx.ctx.alloc(SchemaUnion::new(name));
            ctx.add_type(SchemaType::Union(union));
            union
        }

        #[inline]
        fn on_build(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
            schema_union: &'arena SchemaUnion<'arena>,
        ) {
            for introspection_type_ref in self.possible_types.possible_types.iter() {
                let name = ctx.ctx.alloc_str(introspection_type_ref.name);
                let obj = ctx
                    .get_type(name)
                    .and_then(|named_type| named_type.object());
                if let Some(schema_obj) = obj {
                    schema_union.add_possible_type(&ctx.ctx, schema_obj);
                }
            }
        }
    }

    fn from_input_type_ref<'arena, 'data>(
        ctx: &'arena BuildSchemaContext<'arena>,
        input: &'data IntrospectionInputTypeRef,
    ) -> TypeRef<'arena, InputType<'arena>> {
        use std::ops::Deref;
        match input {
            IntrospectionInputTypeRef::List { of_type } => {
                TypeRef::ListType(ctx.ctx.alloc(from_input_type_ref(ctx, of_type.deref())))
            }
            IntrospectionInputTypeRef::NonNull { of_type } => {
                TypeRef::NonNullType(ctx.ctx.alloc(from_input_type_ref(ctx, of_type.deref())))
            }
            IntrospectionInputTypeRef::ScalarType { name }
            | IntrospectionInputTypeRef::EnumType { name }
            | IntrospectionInputTypeRef::InputObjectType { name } => {
                // TODO: Check whether type matches here
                let name = ctx.ctx.alloc_str(name);
                TypeRef::Type(ctx.get_type(name).and_then(|x| x.input_type()).unwrap())
            }
        }
    }

    fn from_output_type_ref<'arena, 'data>(
        ctx: &'arena BuildSchemaContext<'arena>,
        output: &'data IntrospectionOutputTypeRef,
    ) -> TypeRef<'arena, OutputType<'arena>> {
        use std::ops::Deref;
        match output {
            IntrospectionOutputTypeRef::List { of_type } => {
                TypeRef::ListType(ctx.ctx.alloc(from_output_type_ref(ctx, of_type.deref())))
            }
            IntrospectionOutputTypeRef::NonNull { of_type } => {
                TypeRef::NonNullType(ctx.ctx.alloc(from_output_type_ref(ctx, of_type.deref())))
            }
            IntrospectionOutputTypeRef::ScalarType { name }
            | IntrospectionOutputTypeRef::EnumType { name }
            | IntrospectionOutputTypeRef::ObjectType { name }
            | IntrospectionOutputTypeRef::InterfaceType { name }
            | IntrospectionOutputTypeRef::UnionType { name } => {
                // TODO: Check whether type matches here
                let name = ctx.ctx.alloc_str(name);
                TypeRef::Type(ctx.get_type(name).and_then(|x| x.output_type()).unwrap())
            }
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaObject<'arena>>
        for IntrospectionObjectType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaObject<'arena> {
            let name = ctx.ctx.alloc_str(self.name);
            let object = ctx.ctx.alloc(SchemaObject::new(name));
            ctx.add_type(SchemaType::Object(object));
            object
        }

        #[inline]
        fn on_build(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
            schema_object: &'arena SchemaObject<'arena>,
        ) {
            for field in self.implementation.fields.iter() {
                let field_name = ctx.ctx.alloc_str(field.name);
                let schema_field =
                    SchemaField::new(field_name, from_output_type_ref(ctx, &field.of_type));
                for arg in field.args.iter() {
                    let arg_name = ctx.ctx.alloc_str(arg.name);
                    let input_field =
                        SchemaInputField::new(arg_name, from_input_type_ref(ctx, &arg.of_type));
                    schema_field.add_argument(ctx.ctx, input_field);
                }
                schema_object.add_field(ctx.ctx, schema_field);
            }
            if let Some(interfaces) = &self.implementation.interfaces {
                for introspection_type_ref in interfaces.iter() {
                    let name = ctx.ctx.alloc_str(introspection_type_ref.name);
                    let interface = ctx
                        .get_type(name)
                        .and_then(|named_type| named_type.interface());
                    if let Some(interface) = interface {
                        schema_object.add_interface(&ctx.ctx, interface);
                    }
                }
            }
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaInterface<'arena>>
        for IntrospectionInterfaceType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaInterface<'arena> {
            let name = ctx.ctx.alloc_str(self.name);
            let interface = ctx.ctx.alloc(SchemaInterface::new(name));
            ctx.add_type(SchemaType::Interface(interface));
            interface
        }

        #[inline]
        fn on_build(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
            schema_interface: &'arena SchemaInterface<'arena>,
        ) {
            for field in self.implementation.fields.iter() {
                let field_name = ctx.ctx.alloc_str(field.name);
                let schema_field =
                    SchemaField::new(field_name, from_output_type_ref(ctx, &field.of_type));
                for arg in field.args.iter() {
                    let arg_name = ctx.ctx.alloc_str(arg.name);
                    let input_field =
                        SchemaInputField::new(arg_name, from_input_type_ref(ctx, &arg.of_type));
                    schema_field.add_argument(ctx.ctx, input_field);
                }
                schema_interface.add_field(ctx.ctx, schema_field);
            }
            if let Some(interfaces) = &self.implementation.interfaces {
                for introspection_type_ref in interfaces.iter() {
                    let name = ctx.ctx.alloc_str(introspection_type_ref.name);
                    let interface = ctx
                        .get_type(name)
                        .and_then(|named_type| named_type.interface());
                    if let Some(interface) = interface {
                        schema_interface.add_interface(&ctx.ctx, interface);
                        interface.add_possible_interface(&ctx.ctx, schema_interface);
                    }
                }
            }
            for introspection_type_ref in self.possible_types.possible_types.iter() {
                let name = ctx.ctx.alloc_str(introspection_type_ref.name);
                let obj = ctx
                    .get_type(name)
                    .and_then(|named_type| named_type.object());
                if let Some(schema_obj) = obj {
                    schema_interface.add_possible_type(&ctx.ctx, schema_obj);
                }
            }
        }
    }

    impl<'arena, 'data> BuildSchemaType<'arena, 'data, SchemaInputObject<'arena>>
        for IntrospectionInputObjectType<'data>
    {
        #[inline]
        fn on_create(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
        ) -> &'arena SchemaInputObject<'arena> {
            let name = ctx.ctx.alloc_str(self.name);
            let input = ctx.ctx.alloc(SchemaInputObject::new(name));
            ctx.add_type(SchemaType::InputObject(input));
            input
        }

        #[inline]
        fn on_build(
            &'data self,
            ctx: &'arena BuildSchemaContext<'arena>,
            schema_input: &'arena SchemaInputObject<'arena>,
        ) {
            for field in self.input_fields.iter() {
                let field_name = ctx.ctx.alloc_str(field.name);
                let input_field =
                    SchemaInputField::new(field_name, from_input_type_ref(ctx, &field.of_type));
                schema_input.add_field(ctx.ctx, input_field);
            }
        }
    }
}

pub trait BuildClientSchema<'arena, 'data> {
    /// Converts the introspected data to a [Schema].
    fn build_client_schema(&'data self, ctx: &'arena ASTContext) -> &'arena Schema<'arena>;
}

impl<'arena, 'data> BuildClientSchema<'arena, 'data> for IntrospectionSchema<'data> {
    /// Converts the introspected data to a [Schema].
    fn build_client_schema(&'data self, ctx: &'arena ASTContext) -> &'arena Schema<'arena> {
        let builder_ctx = ctx.alloc(private::BuildSchemaContext::new(ctx));
        builder_ctx.build_schema(self)
    }
}

impl<'arena, 'data> BuildClientSchema<'arena, 'data> for IntrospectionQuery<'data> {
    /// Converts the introspected data to a [Schema].
    fn build_client_schema(&'data self, ctx: &'arena ASTContext) -> &'arena Schema<'arena> {
        self.schema.build_client_schema(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::super::schema::{SchemaFields, SchemaPossibleTypes, SchemaSuperType};
    use super::*;

    #[test]
    fn build_schema() {
        let ctx = ASTContext::new();
        let introspection_json = include_str!("../../fixture/introspection_query.json");
        let introspection: IntrospectionQuery = serde_json::from_str(introspection_json).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let query_root_name = schema.query_type.map(|obj| obj.name).unwrap();
        assert_eq!(query_root_name, "query_root");

        assert!(std::ptr::eq(
            schema
                .get_type(query_root_name)
                .and_then(|t| t.object())
                .unwrap(),
            schema.query_type.unwrap()
        ));
    }

    #[test]
    fn schema_fields() {
        let ctx = ASTContext::new();
        let introspection_json = include_str!("../../fixture/introspection_query.json");
        let introspection: IntrospectionQuery = serde_json::from_str(introspection_json).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let todo_type = schema.get_type("Todo").and_then(|t| t.object()).unwrap();
        let author_type = schema.get_type("Author").unwrap();

        todo_type.get_field("id").unwrap();
        todo_type.get_field("text").unwrap();

        let author_field = todo_type.get_field("author").unwrap();
        let maybe_author_type = author_field.output_type.of_type().into_schema_type();

        assert!(schema.is_sub_type(*author_type, maybe_author_type.into()));
        assert!(*author_type == maybe_author_type);
    }

    #[test]
    fn schema_abstract_relationships() {
        let ctx = ASTContext::new();
        let introspection_json = include_str!("../../fixture/introspection_query.json");
        let introspection: IntrospectionQuery = serde_json::from_str(introspection_json).unwrap();
        let schema = introspection.build_client_schema(&ctx);

        let type_itodo = schema
            .get_type("ITodo")
            .and_then(|t| t.interface())
            .unwrap();
        let type_bigtodo = schema.get_type("BigTodo").and_then(|t| t.object()).unwrap();
        let type_smalltodo = schema
            .get_type("SmallTodo")
            .and_then(|t| t.object())
            .unwrap();
        let type_search = schema
            .get_type("Search")
            .and_then(|t| t.union_type())
            .unwrap();

        assert!(type_search.get_possible_type("SmallTodo").is_some());
        assert!(type_search.get_possible_type("BigTodo").is_some());

        assert!(type_search.is_sub_type(type_smalltodo.into()));
        assert!(type_search.is_sub_type(type_bigtodo.into()));

        assert!(type_itodo.is_sub_type(type_smalltodo.into()));
        assert!(type_itodo.is_sub_type(type_bigtodo.into()));

        assert!(type_itodo.get_possible_type(type_bigtodo.name).is_some());
        assert!(type_itodo.get_possible_type(type_smalltodo.name).is_some());
    }
}

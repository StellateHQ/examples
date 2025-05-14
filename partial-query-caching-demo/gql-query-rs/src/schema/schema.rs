use crate::ast::{ASTContext, OperationKind};
use std::ptr::eq;
use toolshed::{list::List, map::Map, set::Set};

/// Schema Definition
///
/// A schema is created from root types for each kind of operation and is then used against
/// AST documents for validation and execution. In this library the schema is never executable and
/// serves only for metadata and type information. It is hence a "Client Schema".
/// [Reference](https://spec.graphql.org/October2021/#sec-Schema)
#[derive(Debug, Clone, Copy, Default)]
pub struct Schema<'a> {
    pub(crate) query_type: Option<&'a SchemaObject<'a>>,
    pub(crate) mutation_type: Option<&'a SchemaObject<'a>>,
    pub(crate) subscription_type: Option<&'a SchemaObject<'a>>,
    pub(crate) types: Map<'a, &'a str, &'a SchemaType<'a>>,
}

impl<'a> Schema<'a> {
    /// Returns whether the schema is a default, empty schema
    pub fn is_empty(&'a self) -> bool {
        self.types.is_empty()
            && self.query_type.is_none()
            && self.mutation_type.is_none()
            && self.subscription_type.is_none()
    }

    /// Returns the root object type for query operations
    #[inline]
    pub fn query_type(&'a self) -> Option<&'a SchemaObject<'a>> {
        self.query_type
    }

    /// Returns the root object type for mutation operations
    #[inline]
    pub fn mutation_type(&'a self) -> Option<&'a SchemaObject<'a>> {
        self.mutation_type
    }

    /// Returns the root object type for subscription operations
    #[inline]
    pub fn subscription_type(&'a self) -> Option<&'a SchemaObject<'a>> {
        self.subscription_type
    }

    /// Returns the appropriate object type depending on the passed operation kind
    #[inline]
    pub fn get_root_type(&'a self, operation_kind: OperationKind) -> Option<&'a SchemaObject<'a>> {
        match operation_kind {
            OperationKind::Query => self.query_type,
            OperationKind::Mutation => self.mutation_type,
            OperationKind::Subscription => self.subscription_type,
        }
    }

    /// Retrieves a kind by name from known schema types.
    #[inline]
    pub fn get_type(&'a self, name: &'a str) -> Option<&'a SchemaType<'a>> {
        self.types.get(name)
    }

    /// Checks whether a given type is a sub type of another.
    ///
    /// This is typically used for return types of fields. A return type may be any given sub type
    /// of the return type of said field.
    pub fn is_sub_type(&'a self, abstract_type: SchemaType<'a>, sub_type: SchemaType<'a>) -> bool {
        match abstract_type {
            SchemaType::Union(schema_union) => schema_union.is_sub_type(sub_type),
            SchemaType::Interface(schema_interface) => schema_interface.is_sub_type(sub_type),
            SchemaType::Object(schema_object) => {
                if let SchemaType::Object(sub_object_type) = sub_type {
                    sub_object_type == schema_object
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Generic trait for any schema type that implements fields
pub trait SchemaFields<'a>: Sized + Copy {
    /// Add a new [SchemaField] to the list of fields
    fn add_field(&self, ctx: &'a ASTContext, field: SchemaField<'a>);

    /// Get a [Map] of all fields
    fn get_fields(&'a self) -> Map<'a, &'a str, &'a SchemaField<'a>>;

    /// Get a known field by name
    fn get_field(&'a self, name: &'a str) -> Option<&'a SchemaField<'a>> {
        self.get_fields().get(name)
    }
}

/// Generic trait for any schema type that implements interfaces
pub trait SchemaInterfaces<'a>: Sized + Copy {
    /// Add a new [SchemaInterface] to the list of implemented interfaces
    fn add_interface(&'a self, ctx: &'a ASTContext, interface: &'a SchemaInterface<'a>);

    /// Get list of implemented [SchemaInterface]s
    fn get_interfaces(&'a self) -> List<'a, &'a SchemaInterface<'a>>;

    /// Get a specific possible type by name if it exists on the type
    #[inline]
    fn get_interface(&'a self, name: &'a str) -> Option<&'a SchemaInterface<'a>> {
        for interface in self.get_interfaces() {
            if interface.name == name {
                return Some(interface);
            }
        }
        return None;
    }

    /// Checks whether given [ObjectType] is a possible subtype
    #[inline]
    fn implements_interface(&'a self, schema_interface: &'a SchemaInterface<'a>) -> bool {
        self.get_interfaces()
            .into_iter()
            .any(|&interface| interface == schema_interface)
    }
}

/// Generic trait for any schema type that implements interfaces
pub trait SchemaPossibleTypes<'a>: Sized + Copy {
    /// Add a new [SchemaObject] to the list of possible types
    fn add_possible_type(&'a self, ctx: &'a ASTContext, object: &'a SchemaObject<'a>);

    /// Get list of possible [SchemaObject] types
    fn get_possible_types(&'a self) -> List<'a, &'a SchemaObject<'a>>;

    /// Get a specific possible type by name if it exists on the type
    #[inline]
    fn get_possible_type(&'a self, name: &'a str) -> Option<&'a SchemaObject<'a>> {
        for possible_type in self.get_possible_types() {
            if possible_type.name == name {
                return Some(possible_type);
            }
        }
        return None;
    }

    /// Checks whether given [ObjectType] is a possible subtype
    #[inline]
    fn is_possible_type(&'a self, schema_object: &'a SchemaObject<'a>) -> bool {
        self.get_possible_types()
            .into_iter()
            .any(|&possible_type| possible_type == schema_object)
    }
}

/// Generic trait for any schema type that may be a super type of other types
pub trait SchemaSuperType<'a>: Sized + Copy {
    /// Checks whether a given type is a sub type of the current super type.
    fn is_sub_type(&'a self, subtype: SchemaType<'a>) -> bool;
}

/// An Object type definition.
///
/// Most types in GraphQL are objects and define a set of fields and the interfaces they implement.
/// [Reference](https://spec.graphql.org/October2021/#sec-Objects)
#[derive(Debug, Clone, Copy)]
pub struct SchemaObject<'a> {
    pub name: &'a str,
    fields: Map<'a, &'a str, &'a SchemaField<'a>>,
    interfaces: List<'a, &'a SchemaInterface<'a>>,
}

impl<'a> PartialEq for SchemaObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        eq(self, other) || self.name == other.name
    }
}

impl<'a> SchemaObject<'a> {
    #[inline]
    pub fn new(name: &'a str) -> Self {
        SchemaObject {
            name,
            fields: Map::default(),
            interfaces: List::empty(),
        }
    }
}

impl<'a> SchemaFields<'a> for SchemaObject<'a> {
    /// Add a new [SchemaField] to the list of fields
    fn add_field(&self, ctx: &'a ASTContext, field: SchemaField<'a>) {
        self.fields.insert(&ctx.arena, field.name, ctx.alloc(field));
    }

    /// Get a [Map] of all fields on the [SchemaObject]
    fn get_fields(&'a self) -> Map<'a, &'a str, &'a SchemaField<'a>> {
        self.fields
    }
}

impl<'a> SchemaInterfaces<'a> for SchemaObject<'a> {
    /// Add a new [SchemaInterface] to the list of implemented interfaces
    #[inline]
    fn add_interface(&'a self, ctx: &'a ASTContext, interface: &'a SchemaInterface<'a>) {
        self.interfaces.prepend(&ctx.arena, interface);
    }

    /// Get list of implemented [SchemaInterface]s
    #[inline]
    fn get_interfaces(&'a self) -> List<'a, &'a SchemaInterface<'a>> {
        self.interfaces
    }
}

/// An Interface type definition.
///
/// Any object or other interfaces may implement one or more interfaces and must then adhere to the
/// definition of this interface. A field that returns an interface as its return type may return
/// any object that implements this interface.
/// [Reference](https://spec.graphql.org/October2021/#sec-Interfaces)
#[derive(Debug, Clone, Copy)]
pub struct SchemaInterface<'a> {
    pub name: &'a str,
    fields: Map<'a, &'a str, &'a SchemaField<'a>>,
    interfaces: List<'a, &'a SchemaInterface<'a>>,
    possible_interfaces: List<'a, &'a SchemaInterface<'a>>,
    possible_types: List<'a, &'a SchemaObject<'a>>,
}

impl<'a> PartialEq for SchemaInterface<'a> {
    fn eq(&self, other: &Self) -> bool {
        eq(self, other) || self.name == other.name
    }
}

impl<'a> SchemaInterface<'a> {
    #[inline]
    pub fn new(name: &'a str) -> Self {
        SchemaInterface {
            name,
            fields: Map::default(),
            interfaces: List::empty(),
            possible_interfaces: List::empty(),
            possible_types: List::empty(),
        }
    }

    /// Add a new [SchemaInterface] to the list that implements this [SchemaInterface]
    #[inline]
    pub fn add_possible_interface(
        &'a self,
        ctx: &'a ASTContext,
        interface: &'a SchemaInterface<'a>,
    ) {
        self.possible_interfaces.prepend(&ctx.arena, interface);
    }

    /// Get list of possible [SchemaInterface]s that implement this [SchemaInterface]
    #[inline]
    pub fn get_possible_interfaces(&'a self) -> List<'a, &'a SchemaInterface<'a>> {
        self.possible_interfaces
    }

    /// Get a specific possible [SchemaInterface] by name that implements this [SchemaInterface]
    #[inline]
    pub fn get_possible_interface(&'a self, name: &'a str) -> Option<&'a SchemaInterface<'a>> {
        for possible_interface in self.get_possible_interfaces() {
            if possible_interface.name == name {
                return Some(possible_interface);
            }
        }
        return None;
    }
}

impl<'a> SchemaFields<'a> for SchemaInterface<'a> {
    /// Add a new [SchemaField] to the list of fields
    fn add_field(&self, ctx: &'a ASTContext, field: SchemaField<'a>) {
        self.fields.insert(&ctx.arena, field.name, ctx.alloc(field));
    }

    /// Get a [Map] of all fields on the [SchemaInterface]
    fn get_fields(&'a self) -> Map<'a, &'a str, &'a SchemaField<'a>> {
        self.fields
    }
}

impl<'a> SchemaInterfaces<'a> for SchemaInterface<'a> {
    /// Add a new [SchemaInterface] to the list of implemented interfaces
    #[inline]
    fn add_interface(&'a self, ctx: &'a ASTContext, interface: &'a SchemaInterface<'a>) {
        self.interfaces.prepend(&ctx.arena, interface);
    }

    /// Get list of implemented [SchemaInterface]s
    #[inline]
    fn get_interfaces(&'a self) -> List<'a, &'a SchemaInterface<'a>> {
        self.interfaces
    }
}

impl<'a> SchemaPossibleTypes<'a> for SchemaInterface<'a> {
    /// Add a new [SchemaObject] to the list of possible types
    #[inline]
    fn add_possible_type(&'a self, ctx: &'a ASTContext, object: &'a SchemaObject<'a>) {
        self.possible_types.prepend(&ctx.arena, object);
    }

    /// Get list of possible [SchemaObject] types
    #[inline]
    fn get_possible_types(&'a self) -> List<'a, &'a SchemaObject<'a>> {
        self.possible_types
    }
}

impl<'a> SchemaSuperType<'a> for SchemaInterface<'a> {
    #[inline]
    fn is_sub_type(&'a self, sub_type: SchemaType<'a>) -> bool {
        match sub_type {
            SchemaType::Object(schema_object) => schema_object.implements_interface(self),
            SchemaType::Interface(schema_interface) => schema_interface.implements_interface(self),
            _ => false,
        }
    }
}

/// An object Field type definition.
///
/// A field is like a function that given its arguments as input values produces an output value.
/// [Reference](https://spec.graphql.org/October2021/#FieldsDefinition)
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SchemaField<'a> {
    pub name: &'a str,
    pub arguments: Map<'a, &'a str, SchemaInputField<'a>>,
    pub output_type: TypeRef<'a, OutputType<'a>>,
}

impl<'a> SchemaField<'a> {
    #[inline]
    pub fn new(name: &'a str, output_type: TypeRef<'a, OutputType<'a>>) -> Self {
        SchemaField {
            name,
            arguments: Map::default(),
            output_type,
        }
    }

    #[inline]
    pub fn add_argument(&self, ctx: &'a ASTContext, arg: SchemaInputField<'a>) {
        self.arguments.insert(&ctx.arena, arg.name, arg);
    }

    #[inline]
    pub fn get_argument(&self, name: &'a str) -> Option<SchemaInputField<'a>> {
        self.arguments.get(name)
    }
}

/// A Union type definition.
///
/// A union contains a list of possible types that can be returned in its stead when its defined as
/// an output type.
/// [Reference](https://spec.graphql.org/October2021/#sec-Unions)
#[derive(Debug, Clone, Copy)]
pub struct SchemaUnion<'a> {
    pub name: &'a str,
    possible_types: List<'a, &'a SchemaObject<'a>>,
}

impl<'a> PartialEq for SchemaUnion<'a> {
    fn eq(&self, other: &Self) -> bool {
        eq(self, other) || self.name == other.name
    }
}

impl<'a> SchemaUnion<'a> {
    #[inline]
    pub fn new(name: &'a str) -> Self {
        SchemaUnion {
            name,
            possible_types: List::empty(),
        }
    }

    #[inline]
    pub fn is_sub_type(&'a self, sub_type: SchemaType<'a>) -> bool {
        match sub_type {
            SchemaType::Object(schema_object) => self
                .possible_types
                .iter()
                .any(|possible| possible.name == schema_object.name),
            _ => false,
        }
    }
}

impl<'a> SchemaPossibleTypes<'a> for SchemaUnion<'a> {
    /// Add a new [SchemaObject] to the list of possible types
    #[inline]
    fn add_possible_type(&'a self, ctx: &'a ASTContext, object: &'a SchemaObject<'a>) {
        self.possible_types.prepend(&ctx.arena, object);
    }

    /// Get list of possible [SchemaObject] types
    #[inline]
    fn get_possible_types(&'a self) -> List<'a, &'a SchemaObject<'a>> {
        self.possible_types
    }
}

impl<'a> SchemaSuperType<'a> for SchemaUnion<'a> {
    #[inline]
    fn is_sub_type(&'a self, sub_type: SchemaType<'a>) -> bool {
        if let SchemaType::Object(schema_object) = sub_type {
            self.is_possible_type(schema_object)
        } else {
            false
        }
    }
}

/// A Scalar type definition.
///
/// Scalars represent primitive leaf values in GraphQL that are represented with a specific
/// serializer and deserializer, which makes the values returnable to a GraphQL client or readable
/// by a GraphQL API.
/// [Reference](https://spec.graphql.org/October2021/#sec-Scalars)
#[derive(Debug, Clone, Copy)]
pub struct SchemaScalar<'a> {
    pub name: &'a str,
}

impl<'a> PartialEq for SchemaScalar<'a> {
    fn eq(&self, other: &Self) -> bool {
        eq(self, other) || self.name == other.name
    }
}

impl<'a> SchemaScalar<'a> {
    #[inline]
    pub fn new(name: &'a str) -> Self {
        SchemaScalar { name }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SchemaEnum<'a> {
    pub name: &'a str,
    pub values: Set<'a, &'a str>,
}

impl<'a> SchemaEnum<'a> {
    #[inline]
    pub fn new(name: &'a str) -> Self {
        SchemaEnum {
            name,
            values: Set::default(),
        }
    }

    #[inline]
    pub fn add_value(&'a self, ctx: &'a ASTContext, value: &'a str) {
        self.values.insert(&ctx.arena, value);
    }
}

/// An Input Object type definition.
///
/// Inputs, such as arguments, may sometimes be nested and accept objects that must adhere to the
/// shape of an Input Object definition. This is often used to represent more complex inputs.
/// [Reference](https://spec.graphql.org/October2021/#sec-Input-Objects)
#[derive(Debug, Clone, Copy)]
pub struct SchemaInputObject<'a> {
    pub name: &'a str,
    pub fields: Map<'a, &'a str, SchemaInputField<'a>>,
}

impl<'a> PartialEq for SchemaInputObject<'a> {
    fn eq(&self, other: &Self) -> bool {
        eq(self, other) || self.name == other.name
    }
}

impl<'a> SchemaInputObject<'a> {
    #[inline]
    pub fn new(name: &'a str) -> Self {
        SchemaInputObject {
            name,
            fields: Map::default(),
        }
    }

    #[inline]
    pub fn add_field(&'a self, ctx: &'a ASTContext, field: SchemaInputField<'a>) {
        self.fields.insert(&ctx.arena, field.name, field);
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SchemaInputField<'a> {
    pub name: &'a str,
    pub input_type: TypeRef<'a, InputType<'a>>,
}

impl<'a> SchemaInputField<'a> {
    #[inline]
    pub fn new(name: &'a str, input_type: TypeRef<'a, InputType<'a>>) -> Self {
        SchemaInputField { name, input_type }
    }
}

/// A named type enum that represents all possible GraphQL definition types.
///
/// [Reference](https://spec.graphql.org/October2021/#sec-Types)
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SchemaType<'a> {
    InputObject(&'a SchemaInputObject<'a>),
    Object(&'a SchemaObject<'a>),
    Union(&'a SchemaUnion<'a>),
    Interface(&'a SchemaInterface<'a>),
    Scalar(&'a SchemaScalar<'a>),
    Enum(&'a SchemaEnum<'a>),
}

impl<'a> SchemaType<'a> {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            SchemaType::InputObject(x) => x.name,
            SchemaType::Object(x) => x.name,
            SchemaType::Union(x) => x.name,
            SchemaType::Interface(x) => x.name,
            SchemaType::Scalar(x) => x.name,
            SchemaType::Enum(x) => x.name,
        }
    }

    pub fn object(&'a self) -> Option<&'a SchemaObject<'a>> {
        match self {
            SchemaType::Object(x) => Some(x),
            _ => None,
        }
    }

    pub fn interface(&'a self) -> Option<&'a SchemaInterface<'a>> {
        match self {
            SchemaType::Interface(x) => Some(x),
            _ => None,
        }
    }

    pub fn union_type(&'a self) -> Option<&'a SchemaUnion<'a>> {
        match self {
            SchemaType::Union(x) => Some(x),
            _ => None,
        }
    }

    pub fn input_type(&'a self) -> Option<InputType<'a>> {
        match self {
            SchemaType::Scalar(x) => Some(InputType::Scalar(x)),
            SchemaType::Enum(x) => Some(InputType::Enum(x)),
            SchemaType::InputObject(x) => Some(InputType::InputObject(x)),
            _ => None,
        }
    }

    pub fn output_type(&'a self) -> Option<OutputType<'a>> {
        match self {
            SchemaType::Object(x) => Some(OutputType::Object(x)),
            SchemaType::Union(x) => Some(OutputType::Union(x)),
            SchemaType::Interface(x) => Some(OutputType::Interface(x)),
            SchemaType::Scalar(x) => Some(OutputType::Scalar(x)),
            SchemaType::Enum(x) => Some(OutputType::Enum(x)),
            _ => None,
        }
    }
}

impl<'a> From<&'a SchemaObject<'a>> for SchemaType<'a> {
    #[inline]
    fn from(schema_object: &'a SchemaObject<'a>) -> Self {
        SchemaType::Object(schema_object)
    }
}

impl<'a> From<&'a SchemaUnion<'a>> for SchemaType<'a> {
    #[inline]
    fn from(schema_union: &'a SchemaUnion<'a>) -> Self {
        SchemaType::Union(schema_union)
    }
}

impl<'a> From<&'a SchemaInterface<'a>> for SchemaType<'a> {
    #[inline]
    fn from(schema_interface: &'a SchemaInterface<'a>) -> Self {
        SchemaType::Interface(schema_interface)
    }
}

impl<'a> From<OutputType<'a>> for SchemaType<'a> {
    #[inline]
    fn from(type_ref: OutputType<'a>) -> Self {
        match type_ref {
            OutputType::Object(x) => SchemaType::Object(x),
            OutputType::Union(x) => SchemaType::Union(x),
            OutputType::Interface(x) => SchemaType::Interface(x),
            OutputType::Scalar(x) => SchemaType::Scalar(x),
            OutputType::Enum(x) => SchemaType::Enum(x),
        }
    }
}

impl<'a> From<InputType<'a>> for SchemaType<'a> {
    #[inline]
    fn from(type_ref: InputType<'a>) -> Self {
        match type_ref {
            InputType::InputObject(x) => SchemaType::InputObject(x),
            InputType::Scalar(x) => SchemaType::Scalar(x),
            InputType::Enum(x) => SchemaType::Enum(x),
        }
    }
}

/// An output type enum that represents all possible GraphQL definition types that a field may
/// return.
///
/// [Reference](https://spec.graphql.org/October2021/#sec-Input-and-Output-Types)
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OutputType<'a> {
    Object(&'a SchemaObject<'a>),
    Union(&'a SchemaUnion<'a>),
    Interface(&'a SchemaInterface<'a>),
    Scalar(&'a SchemaScalar<'a>),
    Enum(&'a SchemaEnum<'a>),
}

impl<'a> OutputType<'a> {
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            OutputType::Object(x) => x.name,
            OutputType::Union(x) => x.name,
            OutputType::Interface(x) => x.name,
            OutputType::Scalar(x) => x.name,
            OutputType::Enum(x) => x.name,
        }
    }

    #[inline]
    pub fn into_schema_type(&'a self) -> SchemaType<'a> {
        match self {
            OutputType::Object(x) => SchemaType::Object(x),
            OutputType::Union(x) => SchemaType::Union(x),
            OutputType::Interface(x) => SchemaType::Interface(x),
            OutputType::Scalar(x) => SchemaType::Scalar(x),
            OutputType::Enum(x) => SchemaType::Enum(x),
        }
    }
}

/// An input type enum that represents all possible GraphQL definition types that an argument or
/// input object field may accept.
///
/// [Reference](https://spec.graphql.org/October2021/#sec-Input-and-Output-Types)
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputType<'a> {
    InputObject(&'a SchemaInputObject<'a>),
    Scalar(&'a SchemaScalar<'a>),
    Enum(&'a SchemaEnum<'a>),
}

impl<'a> InputType<'a> {
    #[inline]
    pub fn named_type(&'a self) -> SchemaType<'a> {
        match self {
            InputType::InputObject(x) => SchemaType::InputObject(x),
            InputType::Scalar(x) => SchemaType::Scalar(x),
            InputType::Enum(x) => SchemaType::Enum(x),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TypeRef<'a, T: Into<SchemaType<'a>>> {
    Type(T),
    ListType(&'a TypeRef<'a, T>),
    NonNullType(&'a TypeRef<'a, T>),
}

impl<'a, T: Into<SchemaType<'a>> + 'a> TypeRef<'a, T> {
    #[inline]
    pub fn of_type(&'a self) -> &'a T {
        match self {
            TypeRef::Type(of_type) => of_type,
            TypeRef::ListType(of_type) => of_type.of_type(),
            TypeRef::NonNullType(of_type) => of_type.of_type(),
        }
    }
}

use std::{hash, sync::Arc};

use apollo_parser::{ast, SyntaxNode};
use indexmap::IndexMap;

pub mod argument;
pub mod directive;
pub mod enum_;
pub mod field;
pub mod fragment;
pub mod input;
pub mod interface;
pub mod name;
pub mod object;
pub mod operation;
pub mod scalar;
pub mod schema;
pub mod selection;
pub mod ty;
pub mod type_system;
pub mod union_;
pub mod value;
pub mod variable;

pub use argument::{Argument, ArgumentsDefinition};
pub use directive::{Directive, DirectiveDefinition, DirectiveLocation};
pub use enum_::{EnumTypeDefinition, EnumTypeExtension, EnumValueDefinition};
pub use field::{Alias, Field, FieldDefinition};
pub use fragment::FragmentDefinition;
pub use input::{InputObjectTypeDefinition, InputObjectTypeExtension, InputValueDefinition};
pub use interface::{ImplementsInterface, InterfaceTypeDefinition, InterfaceTypeExtension};
pub use name::Name;
pub use object::{ObjectTypeDefinition, ObjectTypeExtension};
pub use operation::{OperationDefinition, OperationType};
pub use scalar::{ScalarTypeDefinition, ScalarTypeExtension};
pub use schema::{RootOperationTypeDefinition, SchemaDefinition, SchemaExtension};
pub use selection::{FragmentSelection, FragmentSpread, InlineFragment, Selection, SelectionSet};
pub use ty::Type;
pub use type_system::{TypeDefinition, TypeExtension, TypeSystem, TypeSystemDefinitions};
pub use union_::{UnionMember, UnionTypeDefinition, UnionTypeExtension};
pub use value::{DefaultValue, Float, Value, Variable};
pub use variable::VariableDefinition;

pub(crate) use schema::RootOperationNames;

use crate::FileId;

pub type ByName<T> = Arc<IndexMap<String, Arc<T>>>;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HirNodeLocation {
    pub(crate) offset: usize,
    pub(crate) node_len: usize,
    pub(crate) file_id: FileId,
}

impl HirNodeLocation {
    pub(crate) fn new(file_id: FileId, node: &'_ SyntaxNode) -> Self {
        let text_range = node.text_range();
        Self {
            offset: text_range.start().into(),
            node_len: text_range.len().into(),
            file_id,
        }
    }

    /// Get file id of the current node.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Get source offset of the current node.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the source offset of the end of the current node.
    pub fn end_offset(&self) -> usize {
        self.offset + self.node_len
    }

    /// Get node length.
    pub fn node_len(&self) -> usize {
        self.node_len
    }
}

impl<Ast: ast::AstNode> From<(FileId, &'_ Ast)> for HirNodeLocation {
    fn from((file_id, node): (FileId, &'_ Ast)) -> Self {
        Self::new(file_id, node.syntax())
    }
}

/// This pre-computes where to find items such as fields of an object type on a
/// type extension based on the item's name.
#[derive(Clone, Debug, Eq)]
pub(crate) struct ByNameWithExtensions {
    /// `(None, i)` designates `def.example[i]`.
    /// `(Some(j), i)` designates `def.extensions[j].example[i]`.
    indices: IndexMap<String, (Option<usize>, usize)>,
}

/// Equivalent to ignoring a `ByNameWithExtensions` field in `PartialEq` for its parent struct,
/// since it is determined by other fields.
impl PartialEq for ByNameWithExtensions {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Equivalent to ignoring a `ByNameWithExtensions` field in `Hash` for its parent struct,
/// since it is determined by other fields.
impl hash::Hash for ByNameWithExtensions {
    fn hash<H: hash::Hasher>(&self, _state: &mut H) {
        // do nothing
    }
}

impl ByNameWithExtensions {
    pub(crate) fn new<Item>(self_items: &[Item], name: impl Fn(&Item) -> &str) -> Self {
        let mut indices = IndexMap::new();
        for (i, item) in self_items.iter().enumerate() {
            indices.entry(name(item).to_owned()).or_insert((None, i));
        }
        ByNameWithExtensions { indices }
    }

    pub(crate) fn add_extension<Item>(
        &mut self,
        extension_index: usize,
        extension_items: &[Item],
        name: impl Fn(&Item) -> &str,
    ) {
        for (i, item) in extension_items.iter().enumerate() {
            self.indices
                .entry(name(item).to_owned())
                .or_insert((Some(extension_index), i));
        }
    }

    fn get_by_index<'a, Item, Ext>(
        &self,
        (ext, i): (Option<usize>, usize),
        self_items: &'a [Item],
        extensions: &'a [Arc<Ext>],
        extension_items: impl Fn(&'a Ext) -> &'a [Item],
    ) -> &'a Item {
        let items = if let Some(j) = ext {
            extension_items(&extensions[j])
        } else {
            self_items
        };
        &items[i]
    }

    pub(crate) fn get<'a, Item, Ext>(
        &self,
        name: &str,
        self_items: &'a [Item],
        extensions: &'a [Arc<Ext>],
        extension_items: impl Fn(&'a Ext) -> &'a [Item],
    ) -> Option<&'a Item> {
        let index = *self.indices.get(name)?;
        Some(self.get_by_index(index, self_items, extensions, extension_items))
    }

    pub(crate) fn iter<'a, Item, Ext>(
        &'a self,
        self_items: &'a [Item],
        extensions: &'a [Arc<Ext>],
        extension_items: impl Fn(&'a Ext) -> &'a [Item] + Copy + 'a,
    ) -> impl Iterator<Item = &'a Item> + ExactSizeIterator + DoubleEndedIterator {
        self.indices
            .values()
            .map(move |&index| self.get_by_index(index, self_items, extensions, extension_items))
    }
}

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::HirDatabase;

    #[test]
    fn extensions() {
        let mut compiler = ApolloCompiler::new();
        let first = r#"
            scalar Scalar @specifiedBy(url: "https://apollographql.com")
            type Object implements Intf {
                field: Int,
            }
            type Object2 {
                field: String,
            }
            interface Intf {
                field: Int,
            }
            input Input {
                field: Enum,
            }
            enum Enum {
                VALUE,
            }
            union Union = Object | Object2;
        "#;
        let second = r#"
            extend scalar Scalar @deprecated(reason: "do something else")
            extend interface Intf implements Intf2 {
                field2: Scalar,
            }
            interface Intf2 {
                field3: String,
            }
            extend type Object implements Intf2 {
                field2: Scalar,
                field3: String,
            }
            extend enum Enum {
                "like VALUE, but more"
                VALUE_2,
            }
            extend input Input {
                field2: Int,
            }
            extend union Union = Query;
            type Query {
                object: Object,
            }
        "#;
        compiler.add_type_system(first, "first.graphql");
        compiler.add_type_system(second, "second.graphql");

        let scalar = &compiler.db.types_definitions_by_name()["Scalar"];
        let object = &compiler.db.object_types()["Object"];
        let interface = &compiler.db.interfaces()["Intf"];
        let input = &compiler.db.input_objects()["Input"];
        let enum_ = &compiler.db.enums()["Enum"];
        let union_ = &compiler.db.unions()["Union"];

        assert_eq!(
            scalar
                .self_directives()
                .iter()
                .map(|d| d.name())
                .collect::<Vec<_>>(),
            ["specifiedBy"]
        );
        assert_eq!(
            scalar.directives().map(|d| d.name()).collect::<Vec<_>>(),
            ["specifiedBy", "deprecated"]
        );
        // assert_eq!(
        //     *scalar
        //         .directive_by_name("deprecated")
        //         .unwrap()
        //         .argument_by_name("reason")
        //         .unwrap(),
        //     super::Value::String("do something else".to_owned())
        // );
        assert!(scalar.directive_by_name("haunted").is_none());

        assert_eq!(
            object
                .self_fields()
                .iter()
                .map(|f| f.name())
                .collect::<Vec<_>>(),
            ["field"]
        );
        assert_eq!(
            object.fields().map(|f| f.name()).collect::<Vec<_>>(),
            ["field", "field2", "field3"]
        );
        assert_eq!(
            object.field(&compiler.db, "field").unwrap().ty().name(),
            "Int"
        );
        assert!(object.field(&compiler.db, "field4").is_none());

        assert_eq!(
            object
                .self_implements_interfaces()
                .iter()
                .map(|i| i.interface())
                .collect::<Vec<_>>(),
            ["Intf"]
        );
        assert_eq!(
            object
                .implements_interfaces()
                .map(|f| f.interface())
                .collect::<Vec<_>>(),
            ["Intf", "Intf2"]
        );
        assert!(object.implements_interface("Intf2"));
        assert!(!object.implements_interface("Intf3"));

        assert_eq!(
            interface
                .self_fields()
                .iter()
                .map(|f| f.name())
                .collect::<Vec<_>>(),
            ["field"]
        );
        assert_eq!(
            interface.fields().map(|f| f.name()).collect::<Vec<_>>(),
            ["field", "field2"]
        );
        assert_eq!(interface.field("field").unwrap().ty().name(), "Int");
        assert!(interface.field("field4").is_none());

        assert!(interface.self_implements_interfaces().is_empty());
        assert_eq!(
            interface
                .implements_interfaces()
                .map(|f| f.interface())
                .collect::<Vec<_>>(),
            ["Intf2"]
        );
        assert!(interface.implements_interface("Intf2"));
        assert!(!interface.implements_interface("Intf3"));

        assert_eq!(
            input
                .self_fields()
                .iter()
                .map(|f| f.name())
                .collect::<Vec<_>>(),
            ["field"]
        );
        assert_eq!(
            input.fields().map(|f| f.name()).collect::<Vec<_>>(),
            ["field", "field2"]
        );
        assert_eq!(input.field("field").unwrap().ty().name(), "Enum");
        assert!(input.field("field3").is_none());

        assert_eq!(
            enum_
                .self_values()
                .iter()
                .map(|v| v.enum_value())
                .collect::<Vec<_>>(),
            ["VALUE"]
        );
        assert_eq!(
            enum_.values().map(|v| v.enum_value()).collect::<Vec<_>>(),
            ["VALUE", "VALUE_2"]
        );
        assert_eq!(
            enum_.value("VALUE_2").unwrap().description(),
            Some("like VALUE, but more")
        );
        assert!(enum_.value("VALUE_3").is_none());

        assert_eq!(
            union_
                .self_members()
                .iter()
                .map(|m| m.name())
                .collect::<Vec<_>>(),
            ["Object", "Object2"]
        );
        assert_eq!(
            union_.members().map(|m| m.name()).collect::<Vec<_>>(),
            ["Object", "Object2", "Query"]
        );
        assert!(union_.has_member("Object2"));
        assert!(!union_.has_member("Enum"));
    }

    #[test]
    fn query_extended_type() {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system("type Query { foo: String }", "base.graphql");
        compiler.add_type_system("extend type Query { bar: Int }", "ext.graphql");
        compiler.add_executable("{ bar }", "query.graphql");
        let operations = compiler.db.all_operations();
        let fields = operations[0].fields(&compiler.db);
        // This unwrap failed before https://github.com/apollographql/apollo-rs/pull/482
        // changed the behavior of `ObjectTypeDefinition::field(name)` in `hir_db::parent_ty`
        let ty = fields[0].ty(&compiler.db).unwrap();
        assert_eq!(ty.name(), "Int");
    }

    #[test]
    fn syntax_errors() {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(
            "type Person {
                id: ID!
                name: String
                appearedIn: [Film]s
                directed: [Film]
            }",
            "person.graphql",
        );
        let person = compiler
            .db
            .find_object_type_by_name("Person".into())
            .unwrap();
        let hir_field_names: Vec<_> = person
            .fields_definition
            .iter()
            .map(|field| field.name())
            .collect();
        assert_eq!(hir_field_names, ["id", "name", "appearedIn", "directed"]);
    }

    #[test]
    fn built_in_types() {
        let input = r#"
type Query {
  id: String
  name: String
  birthday: Date
}
        "#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(input, "ts.graphql");
        let db = compiler.db;

        // introspection types
        assert!(db
            .find_object_type_by_name("__Schema".to_string())
            .is_some());
        assert!(db.find_object_type_by_name("__Type".to_string()).is_some());
        assert!(db.find_enum_by_name("__TypeKind".to_string()).is_some());
        assert!(db.find_object_type_by_name("__Field".to_string()).is_some());
        assert!(db
            .find_object_type_by_name("__InputValue".to_string())
            .is_some());
        assert!(db
            .find_object_type_by_name("__EnumValue".to_string())
            .is_some());
        assert!(db
            .find_object_type_by_name("__Directive".to_string())
            .is_some());
        assert!(db
            .find_enum_by_name("__DirectiveLocation".to_string())
            .is_some());

        // scalar types
        assert!(db.find_scalar_by_name("Int".to_string()).is_some());
        assert!(db.find_scalar_by_name("Float".to_string()).is_some());
        assert!(db.find_scalar_by_name("Boolean".to_string()).is_some());
        assert!(db.find_scalar_by_name("String".to_string()).is_some());
        assert!(db.find_scalar_by_name("ID".to_string()).is_some());

        // directive definitions
        assert!(db
            .find_directive_definition_by_name("specifiedBy".to_string())
            .is_some());
        assert!(db
            .find_directive_definition_by_name("skip".to_string())
            .is_some());
        assert!(db
            .find_directive_definition_by_name("include".to_string())
            .is_some());
        assert!(db
            .find_directive_definition_by_name("deprecated".to_string())
            .is_some());
    }

    #[test]
    fn built_in_types_in_type_system_hir() {
        let mut compiler_1 = ApolloCompiler::new();
        compiler_1.add_type_system("type Query { unused: Int }", "unused.graphql");

        let mut compiler_2 = ApolloCompiler::new();
        compiler_2.set_type_system_hir(compiler_1.db.type_system());
        assert!(compiler_2
            .db
            .object_types_with_built_ins()
            .contains_key("__Schema"));
        assert!(compiler_2
            .db
            .enums_with_built_ins()
            .contains_key("__TypeKind"));
    }
}

use std::fmt;

use crate::{
    DirectiveDef, EnumDef, FragmentDef, InputObjectDef, InterfaceDef, ObjectDef, OperationDef,
    ScalarDef, SchemaDef, UnionDef,
};

/// The `__Document` type represents a GraphQL document.A GraphQL Document describes a complete file or request string operated on by a GraphQL service or client.
/// A document contains multiple definitions, either executable or representative of a GraphQL type system.
///
/// *Document*:
///     OperationDefinition*
///     FragmentDefinition*
///     SchemaDefinition*
///     ScalarTypeDefinition*
///     ObjectTypeDefinition*
///     InterfaceTypeDefinition*
///     UnionTypeDefinition*
///     EnumTypeDefinition*
///     InputObjectDefinition*
///     DirectiveDefinition*
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Document).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{
///     Argument, Directive, Document, Field, OperationDef, OperationType, Selection, SelectionSet, Type_, Value,
///     VariableDef,
/// };
/// use indoc::indoc;
///
/// let mut document = Document::new();
/// let selection_set = {
///     let sels = vec![
///         Selection::Field(Field::new(String::from("first"))),
///         Selection::Field(Field::new(String::from("second"))),
///     ];
///     let mut sel_set = SelectionSet::new();
///     sels.into_iter().for_each(|sel| sel_set.selection(sel));
///
///     sel_set
/// };
/// let var_def = VariableDef::new(
///     String::from("variable_def"),
///     Type_::List {
///         ty: Box::new(Type_::NamedType {
///             name: String::from("Int"),
///         }),
///     },
/// );
/// let mut new_op = OperationDef::new(OperationType::Query, selection_set);
/// let mut directive = Directive::new(String::from("testDirective"));
/// directive.arg(Argument::new(
///     String::from("first"),
///     Value::String("one".to_string()),
/// ));
/// new_op.variable_definition(var_def);
/// new_op.directive(directive);
///
/// document.operation(new_op);
///
/// assert_eq!(
///     document.to_string(),
///     indoc! { r#"
///         query($variable_def: [Int]) @testDirective(first: "one") {
///           first
///           second
///         }
///     "#}
/// );
/// ```
#[derive(Debug, Default)]
pub struct Document {
    operation_definitions: Vec<OperationDef>,
    fragment_definitions: Vec<FragmentDef>,
    schema_definitions: Vec<SchemaDef>,
    // Type definitions
    scalar_type_definitions: Vec<ScalarDef>,
    object_type_definitions: Vec<ObjectDef>,
    interface_type_definitions: Vec<InterfaceDef>,
    union_type_definitions: Vec<UnionDef>,
    enum_type_definitions: Vec<EnumDef>,
    input_object_type_definitions: Vec<InputObjectDef>,
    // DirectiveDefs
    directive_definitions: Vec<DirectiveDef>,
}

impl Document {
    /// Create a new instance of Document
    pub fn new() -> Self {
        Self::default()
    }

    /// Add operation
    pub fn operation(&mut self, operation_definition: OperationDef) {
        self.operation_definitions.push(operation_definition);
    }

    /// Add fragment
    pub fn fragment(&mut self, fragment_definition: FragmentDef) {
        self.fragment_definitions.push(fragment_definition);
    }

    /// Add schema
    pub fn schema(&mut self, schema_definition: SchemaDef) {
        self.schema_definitions.push(schema_definition);
    }
    /// Add scalar
    pub fn scalar(&mut self, scalar_type_definition: ScalarDef) {
        self.scalar_type_definitions.push(scalar_type_definition);
    }
    /// Add object
    pub fn object(&mut self, object_type_definition: ObjectDef) {
        self.object_type_definitions.push(object_type_definition);
    }
    /// Add interface
    pub fn interface(&mut self, interface_type_definition: InterfaceDef) {
        self.interface_type_definitions
            .push(interface_type_definition);
    }
    /// Add union
    pub fn union(&mut self, union_type_definition: UnionDef) {
        self.union_type_definitions.push(union_type_definition);
    }
    /// Add enum
    pub fn enum_(&mut self, enum_type_definition: EnumDef) {
        self.enum_type_definitions.push(enum_type_definition);
    }

    /// Add input_object
    pub fn input_object_(&mut self, input_object_type_definition: InputObjectDef) {
        self.input_object_type_definitions
            .push(input_object_type_definition);
    }
    /// Add directive
    pub fn directive(&mut self, directive_definition: DirectiveDef) {
        self.directive_definitions.push(directive_definition);
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for operation_def in &self.operation_definitions {
            write!(f, "{}", operation_def)?;
        }
        for fragment_def in &self.fragment_definitions {
            write!(f, "{}", fragment_def)?;
        }
        for schema_def in &self.schema_definitions {
            write!(f, "{}", schema_def)?;
        }
        for scalar_type_def in &self.scalar_type_definitions {
            write!(f, "{}", scalar_type_def)?;
        }
        for object_type_def in &self.object_type_definitions {
            write!(f, "{}", object_type_def)?;
        }
        for interface_type_def in &self.interface_type_definitions {
            write!(f, "{}", interface_type_def)?;
        }
        for union_type_def in &self.union_type_definitions {
            write!(f, "{}", union_type_def)?;
        }
        for enum_type_def in &self.enum_type_definitions {
            write!(f, "{}", enum_type_def)?;
        }
        for union_type_def in &self.union_type_definitions {
            write!(f, "{}", union_type_def)?;
        }
        for input_object_type_def in &self.input_object_type_definitions {
            write!(f, "{}", input_object_type_def)?;
        }
        for directive_def in &self.directive_definitions {
            write!(f, "{}", directive_def)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Argument, Directive, Field, OperationType, Selection, SelectionSet, Type_, Value,
        VariableDef,
    };

    use super::*;
    use indoc::indoc;

    #[test]
    fn it_encodes_a_document_with_operation() {
        let mut document = Document::new();
        let selection_set = {
            let sels = vec![
                Selection::Field(Field::new(String::from("first"))),
                Selection::Field(Field::new(String::from("second"))),
            ];
            let mut sel_set = SelectionSet::new();
            sels.into_iter().for_each(|sel| sel_set.selection(sel));

            sel_set
        };
        let var_def = VariableDef::new(
            String::from("variable_def"),
            Type_::List {
                ty: Box::new(Type_::NamedType {
                    name: String::from("Int"),
                }),
            },
        );
        let mut new_op = OperationDef::new(OperationType::Query, selection_set);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));
        new_op.variable_definition(var_def);
        new_op.directive(directive);

        document.operation(new_op);

        assert_eq!(
            document.to_string(),
            indoc! { r#"
                query($variable_def: [Int]) @testDirective(first: "one") {
                  first
                  second
                }
            "#}
        );
    }
}

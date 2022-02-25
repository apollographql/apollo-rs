use std::fmt;

use crate::{
    DirectiveDefinition, EnumDefinition, FragmentDefinition, InputObjectDefinition,
    InterfaceDefinition, ObjectDefinition, OperationDefinition, ScalarDefinition, SchemaDefinition,
    UnionDefinition,
};

/// The `Document` type represents a GraphQL document. A GraphQL Document
/// describes a complete file or request string operated on by a GraphQL service
/// or client.  A document contains multiple definitions, either executable or
/// representative of a GraphQL type system.
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
///     Argument, Directive, Document, Field, OperationDefinition, OperationType, Selection, SelectionSet, Type_, Value,
///     VariableDefinition,
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
/// let var_def = VariableDefinition::new(
///     String::from("variable_def"),
///     Type_::List {
///         ty: Box::new(Type_::NamedType {
///             name: String::from("Int"),
///         }),
///     },
/// );
/// let mut new_op = OperationDefinition::new(OperationType::Query, selection_set);
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
    operation_definitions: Vec<OperationDefinition>,
    fragment_definitions: Vec<FragmentDefinition>,
    schema_definitions: Vec<SchemaDefinition>,
    // Type definitions
    scalar_type_definitions: Vec<ScalarDefinition>,
    object_type_definitions: Vec<ObjectDefinition>,
    interface_type_definitions: Vec<InterfaceDefinition>,
    union_type_definitions: Vec<UnionDefinition>,
    enum_type_definitions: Vec<EnumDefinition>,
    input_object_type_definitions: Vec<InputObjectDefinition>,
    // DirectiveDefs
    directive_definitions: Vec<DirectiveDefinition>,
}

impl Document {
    /// Create a new instance of Document
    pub fn new() -> Self {
        Self::default()
    }

    /// Add operation
    pub fn operation(&mut self, operation_definition: OperationDefinition) {
        self.operation_definitions.push(operation_definition);
    }

    /// Add fragment
    pub fn fragment(&mut self, fragment_definition: FragmentDefinition) {
        self.fragment_definitions.push(fragment_definition);
    }

    /// Add schema
    pub fn schema(&mut self, schema_definition: SchemaDefinition) {
        self.schema_definitions.push(schema_definition);
    }
    /// Add scalar
    pub fn scalar(&mut self, scalar_type_definition: ScalarDefinition) {
        self.scalar_type_definitions.push(scalar_type_definition);
    }
    /// Add object
    pub fn object(&mut self, object_type_definition: ObjectDefinition) {
        self.object_type_definitions.push(object_type_definition);
    }
    /// Add interface
    pub fn interface(&mut self, interface_type_definition: InterfaceDefinition) {
        self.interface_type_definitions
            .push(interface_type_definition);
    }
    /// Add union
    pub fn union(&mut self, union_type_definition: UnionDefinition) {
        self.union_type_definitions.push(union_type_definition);
    }
    /// Add enum
    pub fn enum_(&mut self, enum_type_definition: EnumDefinition) {
        self.enum_type_definitions.push(enum_type_definition);
    }

    /// Add input_object
    pub fn input_object(&mut self, input_object_type_definition: InputObjectDefinition) {
        self.input_object_type_definitions
            .push(input_object_type_definition);
    }
    /// Add directive
    pub fn directive(&mut self, directive_definition: DirectiveDefinition) {
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
        Argument, Directive, EnumValue, Field, FragmentSpread, InlineFragment, OperationType,
        Selection, SelectionSet, TypeCondition, Type_, Value, VariableDefinition,
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
        let var_def = VariableDefinition::new(
            String::from("variable_def"),
            Type_::List {
                ty: Box::new(Type_::NamedType {
                    name: String::from("Int"),
                }),
            },
        );
        let mut new_op = OperationDefinition::new(OperationType::Query, selection_set);
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

    #[test]
    fn it_encodes_document_with_operation_and_fragments() {
        let mut document = Document::new();

        let mut droid_selection_set = SelectionSet::new();
        let primary_function_field = Selection::Field(Field::new(String::from("primaryFunction")));
        droid_selection_set.selection(primary_function_field);

        let mut droid_inline_fragment = InlineFragment::new(droid_selection_set);
        droid_inline_fragment.type_condition(Some(TypeCondition::new(String::from("Droid"))));

        let comparison_fields_fragment_spread =
            FragmentSpread::new(String::from("comparisonFields"));

        let name_field = Field::new(String::from("name"));

        let hero_selection_set = vec![
            Selection::Field(name_field),
            Selection::FragmentSpread(comparison_fields_fragment_spread),
            Selection::InlineFragment(droid_inline_fragment),
        ];

        let mut hero_field = Field::new(String::from("hero"));
        hero_field.selection_set(Some(SelectionSet::with_selections(hero_selection_set)));

        let hero_for_episode_selection_set = vec![Selection::Field(hero_field)];
        let mut hero_for_episode_operation = OperationDefinition::new(
            OperationType::Query,
            SelectionSet::with_selections(hero_for_episode_selection_set),
        );
        hero_for_episode_operation.name(Some(String::from("HeroForEpisode")));

        document.operation(hero_for_episode_operation);

        assert_eq!(
            document.to_string(),
            indoc! { r#"
                query HeroForEpisode {
                  hero {
                    name
                    ...comparisonFields
                    ... on Droid {
                      primaryFunction
                    }
                  }
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_a_document_with_type_system_definition() {
        let mut document = Document::new();

        // Create a Directive Definition.
        let mut directive_def = DirectiveDefinition::new("provideTreat".to_string());
        directive_def.description(Some("Ensures cats get treats.".to_string()));
        directive_def.location("OBJECT".to_string());
        directive_def.location("FIELD_DEFINITION".to_string());
        directive_def.location("INPUT_FIELD_DEFINITION".to_string());
        document.directive(directive_def);

        // Create an Enum Definition
        let mut enum_ty_1 = EnumValue::new("CatTree".to_string());
        enum_ty_1.description(Some("Top bunk of a cat tree.".to_string()));
        let enum_ty_2 = EnumValue::new("Bed".to_string());
        let mut enum_ty_3 = EnumValue::new("CardboardBox".to_string());
        let mut directive = Directive::new(String::from("deprecated"));
        directive.arg(Argument::new(
            String::from("reason"),
            Value::String("Box was recycled.".to_string()),
        ));
        enum_ty_3.directive(directive);

        let mut enum_def = EnumDefinition::new("NapSpots".to_string());
        enum_def.description(Some("Favourite cat\nnap spots.".to_string()));
        enum_def.value(enum_ty_1);
        enum_def.value(enum_ty_2);
        enum_def.value(enum_ty_3);
        document.enum_(enum_def);
        // Union Definition
        let mut union_def = UnionDefinition::new("Pet".to_string());
        union_def.description(Some(
            "A union of all pets represented within a household.".to_string(),
        ));
        union_def.member("Cat".to_string());
        union_def.member("Dog".to_string());
        document.union(union_def);

        assert_eq!(
            document.to_string(),
            indoc! { r#"
        "A union of all pets represented within a household."
        union Pet = Cat | Dog
        """
        Favourite cat
        nap spots.
        """
        enum NapSpots {
          "Top bunk of a cat tree."
          CatTree
          Bed
          CardboardBox @deprecated(reason: "Box was recycled.")
        }
        "Ensures cats get treats."
        directive @provideTreat on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
    "# }
        );
    }
}

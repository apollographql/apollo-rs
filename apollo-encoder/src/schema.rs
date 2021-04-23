use crate::*;

/// GraphQLSchema represented in Schema Definition Language.
///
/// SDL is used as a human-readable format for a given schema to help define and
/// store schema as a string.
/// More information about SDL can be read in this [documentation](https://www.apollographql.com/docs/apollo-server/schema/schema/)
///
/// The `Schema` struct provides method to encode various types to a schema.
///
/// ### Example
/// ```rust
/// use sdl_encoder::{Schema, Field, UnionDef, EnumValue, Directive, EnumDef, Type_};
/// use indoc::indoc;
///
/// let mut schema = Schema::new();
///
/// let mut union_def = UnionDef::new("Cat".to_string());
/// union_def.description(Some(
///     "A union of all cats represented within a household.".to_string(),
/// ));
/// union_def.member("NORI".to_string());
/// union_def.member("CHASHU".to_string());
/// schema.union(union_def);
/// assert_eq!(
///     schema.finish(),
///     indoc! { r#"
///         """A union of all cats represented within a household."""
///         union Cat = NORI | CHASHU
///     "# }
/// );
/// ```
#[derive(Debug)]
pub struct Schema {
    buf: String,
}

impl Schema {
    /// Creates a new instance of Schema Encoder.
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    /// Adds a new Directive Definition.
    pub fn directive(&mut self, directive: Directive) {
        self.buf.push_str(&directive.to_string());
    }

    /// Adds a new Type Definition.
    pub fn object(&mut self, object: ObjectDef) {
        self.buf.push_str(&object.to_string());
    }

    /// Adds a new Schema Definition.
    ///
    /// The schema type is only used when the root GraphQL type is different
    /// from default GraphQL types.
    pub fn schema(&mut self, schema: SchemaDef) {
        self.buf.push_str(&schema.to_string());
    }

    /// Adds a new Input Object Definition.
    pub fn input(&mut self, input: InputObjectDef) {
        self.buf.push_str(&input.to_string());
    }

    /// Adds a new Enum Definition.
    pub fn enum_(&mut self, enum_: EnumDef) {
        self.buf.push_str(&enum_.to_string());
    }

    /// Adds a new Scalar Definition.
    pub fn scalar(&mut self, scalar: ScalarDef) {
        self.buf.push_str(&scalar.to_string());
    }

    /// Adds a new Union Definition.
    pub fn union(&mut self, union_: UnionDef) {
        self.buf.push_str(&union_.to_string());
    }

    /// Adds a new Interface Definition.
    pub fn interface(&mut self, interface: InterfaceDef) {
        self.buf.push_str(&interface.to_string());
    }

    /// Return the encoded SDL string after all types have been processed.
    pub fn finish(self) -> String {
        self.buf
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_test() {
        let mut schema = Schema::new();

        // create a directive
        let mut directive = Directive::new("provideTreat".to_string());
        directive.description(Some("Ensures cats get treats.".to_string()));
        directive.location("OBJECT".to_string());
        directive.location("FIELD_DEFINITION".to_string());
        directive.location("INPUT_FIELD_DEFINITION".to_string());
        schema.directive(directive);

        // a schema definition
        let mut schema_def = SchemaDef::new();
        schema_def.query("TryingToFindCatQuery".to_string());
        schema.schema(schema_def);

        // create a field
        let field_value = Type_::NamedType {
            name: "String".to_string(),
        };

        let null_field = Type_::NonNull {
            ty: Box::new(field_value),
        };

        let mut field = Field::new("cat".to_string(), null_field);
        field.description(Some("Very good cats".to_string()));

        // Union Definition
        let mut union_def = UnionDef::new("Pet".to_string());
        union_def.description(Some("A union of all animals in a household.".to_string()));
        union_def.member("Cat".to_string());
        union_def.member("Dog".to_string());
        schema.union(union_def);

        // Object Definition.
        let object_value = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let object_value_2 = Type_::List {
            ty: Box::new(object_value),
        };

        let mut object_field = Field::new("toys".to_string(), object_value_2);
        object_field.deprecated(Some("Cats are too spoiled".to_string()));

        let object_value_2 = Type_::NamedType {
            name: "FoodType".to_string(),
        };

        let mut object_field_2 = Field::new("food".to_string(), object_value_2);
        object_field_2.description(Some("Dry or wet food?".to_string()));

        let object_field_3 = Type_::NamedType {
            name: "Boolean".to_string(),
        };
        let object_field_3 = Field::new("catGrass".to_string(), object_field_3);

        let mut object_def = ObjectDef::new("PetStoreTrip".to_string());
        object_def.field(object_field);
        object_def.field(object_field_2);
        object_def.field(object_field_3);
        object_def.interface("ShoppingTrip".to_string());
        schema.object(object_def);

        // Enum definition
        let mut enum_ty_1 = EnumValue::new("CAT_TREE".to_string());
        enum_ty_1.description(Some("Top bunk of a cat tree.".to_string()));
        let enum_ty_2 = EnumValue::new("BED".to_string());
        let mut enum_ty_3 = EnumValue::new("CARDBOARD_BOX".to_string());
        enum_ty_3.deprecated(Some("Box was recycled.".to_string()));

        let mut enum_def = EnumDef::new("NapSpots".to_string());
        enum_def.description(Some("Favourite cat nap spots.".to_string()));
        enum_def.value(enum_ty_1);
        enum_def.value(enum_ty_2);
        enum_def.value(enum_ty_3);
        schema.enum_(enum_def);

        let mut scalar = ScalarDef::new("NumberOfTreatsPerDay".to_string());
        scalar.description(Some(
            "Int representing number of treats received.".to_string(),
        ));
        schema.scalar(scalar);

        // input definition
        let input_value = Type_::NamedType {
            name: "DanglerPoleToys".to_string(),
        };

        let input_value_2 = Type_::List {
            ty: Box::new(input_value),
        };
        let mut input_field = InputField::new("toys".to_string(), input_value_2);
        input_field.default(Some("\"Cat Dangler Pole Bird\"".to_string()));
        let input_value_3 = Type_::NamedType {
            name: "FavouriteSpots".to_string(),
        };
        let mut input_value_2 = InputField::new("playSpot".to_string(), input_value_3);
        input_value_2.description(Some("Best playime spots, e.g. tree, bed.".to_string()));

        let mut input_def = InputObjectDef::new("PlayTime".to_string());
        input_def.field(input_field);
        input_def.field(input_value_2);
        schema.input(input_def);

        assert_eq!(
            schema.finish(),
            indoc! { r#"
                """Ensures cats get treats."""
                directive @provideTreat on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
                schema {
                  query: TryingToFindCatQuery
                }
                """A union of all animals in a household."""
                union Pet = Cat | Dog
                type PetStoreTrip implements ShoppingTrip {
                  toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
                  """Dry or wet food?"""
                  food: FoodType
                  catGrass: Boolean
                }
                """Favourite cat nap spots."""
                enum NapSpots {
                  """Top bunk of a cat tree."""
                  CAT_TREE
                  BED
                  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")
                }
                """Int representing number of treats received."""
                scalar NumberOfTreatsPerDay
                input PlayTime {
                  toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
                  """Best playime spots, e.g. tree, bed."""
                  playSpot: FavouriteSpots
                }
            "# }
        );
    }
}

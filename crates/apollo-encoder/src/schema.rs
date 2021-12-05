use crate::{
    Directive, EnumDef, InputObjectDef, InterfaceDef, ObjectDef, ScalarDef, SchemaDef, UnionDef,
};

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
/// use apollo_encoder::{Schema, Field, UnionDef, EnumValue, Directive, EnumDef, Type_};
/// use indoc::indoc;
///
/// let mut schema = Schema::new();
///
/// let mut union_def = UnionDef::new("Cat");
/// union_def.description("A union of all cats represented within a household.");
/// union_def.member("NORI");
/// union_def.member("CHASHU");
/// schema.union(union_def);
/// assert_eq!(
///     schema.finish(),
///     indoc! { r#"
///         "A union of all cats represented within a household."
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
    use crate::{
        Directive, EnumDef, EnumValue, Field, InputField, InputObjectDef, ObjectDef, ScalarDef,
        Schema, SchemaDef, Type_, UnionDef,
    };
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_test() {
        let mut schema = Schema::new();

        let directive = {
            // create a directive
            let mut directive = Directive::new("provideTreat");
            directive.description("Ensures cats get treats.");
            directive.location("OBJECT");
            directive.location("FIELD_DEFINITION");
            directive.location("INPUT_FIELD_DEFINITION");
            directive
        };
        schema.directive(directive);

        let schema_def = {
            // a schema definition
            let mut schema_def = SchemaDef::new();
            schema_def.query("TryingToFindCatQuery");
            schema_def
        };
        schema.schema(schema_def);

        // Union Definition
        let union_def = {
            let mut union_def = UnionDef::new("Pet");
            union_def.description("A union of all animals in a household.");
            union_def.member("Cat");
            union_def.member("Dog");
            union_def
        };
        schema.union(union_def);

        // Object Definition.
        let object_def = {
            let field_1 = {
                let ty = Type_::named_type("DanglerPoleToys");
                let ty = Type_::list(Box::new(ty));

                let mut field = Field::new("toys", ty);
                field.deprecated("Cats are too spoiled");
                field
            };

            let field_2 = {
                let ty = Type_::named_type("FoodType");

                let mut field = Field::new("food", ty);
                field.description("Dry or wet food?");
                field
            };

            let field_3 = {
                let field = Type_::named_type("Boolean");

                Field::new("catGrass", field)
            };

            let mut object_def = ObjectDef::new("PetStoreTrip");
            object_def.field(field_1);
            object_def.field(field_2);
            object_def.field(field_3);
            object_def.interface("ShoppingTrip");
            object_def
        };
        schema.object(object_def);

        // Enum definition
        let enum_def = {
            let enum_value_1 = {
                let mut enum_value = EnumValue::new("CAT_TREE");
                enum_value.description("Top bunk of a cat tree.");
                enum_value
            };
            let enum_value_2 = EnumValue::new("BED");
            let enum_value_3 = {
                let mut enum_value_3 = EnumValue::new("CARDBOARD_BOX");
                enum_value_3.deprecated("Box was recycled.");
                enum_value_3
            };

            let mut enum_def = EnumDef::new("NapSpots");
            enum_def.description("Favourite cat nap spots.");
            enum_def.value(enum_value_1);
            enum_def.value(enum_value_2);
            enum_def.value(enum_value_3);
            enum_def
        };
        schema.enum_(enum_def);

        let scalar = {
            let mut scalar = ScalarDef::new("NumberOfTreatsPerDay");
            scalar.description("Int representing number of treats received.");
            scalar
        };
        schema.scalar(scalar);

        // input definition
        let input_def = {
            let input_field = {
                let ty = Type_::named_type("DanglerPoleToys");
                let ty = Type_::list(Box::new(ty));
                let mut input_field_1 = InputField::new("toys", ty);
                input_field_1.default("\"Cat Dangler Pole Bird\"");
                input_field_1
            };
            let input_value_2 = {
                let ty = Type_::named_type("FavouriteSpots");
                let mut input_value_2 = InputField::new("playSpot", ty);
                input_value_2.description("Best playime spots, e.g. \"tree\", \"bed\".");
                input_value_2
            };

            let mut input_def = InputObjectDef::new("PlayTime");
            input_def.field(input_field);
            input_def.field(input_value_2);
            input_def
        };
        schema.input(input_def);

        assert_eq!(
            schema.finish(),
            indoc! { r#"
                "Ensures cats get treats."
                directive @provideTreat on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION
                schema {
                  query: TryingToFindCatQuery
                }
                "A union of all animals in a household."
                union Pet = Cat | Dog
                type PetStoreTrip implements ShoppingTrip {
                  toys: [DanglerPoleToys] @deprecated(reason: "Cats are too spoiled")
                  "Dry or wet food?"
                  food: FoodType
                  catGrass: Boolean
                }
                "Favourite cat nap spots."
                enum NapSpots {
                  "Top bunk of a cat tree."
                  CAT_TREE
                  BED
                  CARDBOARD_BOX @deprecated(reason: "Box was recycled.")
                }
                "Int representing number of treats received."
                scalar NumberOfTreatsPerDay
                input PlayTime {
                  toys: [DanglerPoleToys] = "Cat Dangler Pole Bird"
                  """
                  Best playime spots, e.g. "tree", "bed".
                  """
                  playSpot: FavouriteSpots
                }
            "# }
        );
    }
}

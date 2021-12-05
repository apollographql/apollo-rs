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
/// use apollo_encoder::{Schema, FieldBuilder, UnionDefBuilder, EnumValueBuilder, DirectiveBuilder, EnumDefBuilder, Type_};
/// use indoc::indoc;
///
/// let union_def = UnionDefBuilder::new("Cat")
///     .description("A union of all cats represented within a household.")
///     .member("NORI")
///     .member("CHASHU")
///     .build();
///
/// let schema = Schema::new()
///     .union(union_def)
///     .finish();
///
/// assert_eq!(
///     schema,
///     indoc! { r#"
///         "A union of all cats represented within a household."
///         union Cat = NORI | CHASHU
///     "# }
/// );
/// ```
///
#[derive(Debug, Default)]
pub struct Schema {
    buf: String,
}

impl Schema {
    /// Creates a new instance of Schema Encoder.
    pub fn new() -> Self {
        Self { buf: String::new() }
    }

    /// Adds a new Directive Definition.
    pub fn directive(mut self, directive: Directive) -> Self {
        self.buf.push_str(&directive.to_string());
        self
    }

    /// Adds a new Type Definition.
    pub fn object(mut self, object: ObjectDef) -> Self {
        self.buf.push_str(&object.to_string());
        self
    }

    /// Adds a new Schema Definition.
    ///
    /// The schema type is only used when the root GraphQL type is different
    /// from default GraphQL types.
    pub fn schema(mut self, schema: SchemaDef) -> Self {
        self.buf.push_str(&schema.to_string());
        self
    }

    /// Adds a new Input Object Definition.
    pub fn input(mut self, input: InputObjectDef) -> Self {
        self.buf.push_str(&input.to_string());
        self
    }

    /// Adds a new Enum Definition.
    pub fn enum_(mut self, enum_: EnumDef) -> Self {
        self.buf.push_str(&enum_.to_string());
        self
    }

    /// Adds a new Scalar Definition.
    pub fn scalar(mut self, scalar: ScalarDef) -> Self {
        self.buf.push_str(&scalar.to_string());
        self
    }

    /// Adds a new Union Definition.
    pub fn union(mut self, union_: UnionDef) -> Self {
        self.buf.push_str(&union_.to_string());
        self
    }

    /// Adds a new Interface Definition.
    pub fn interface(mut self, interface: InterfaceDef) -> Self {
        self.buf.push_str(&interface.to_string());
        self
    }

    /// Return the encoded SDL string after all types have been processed.
    pub fn finish(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DirectiveBuilder, EnumDefBuilder, EnumValueBuilder, FieldBuilder, InputFieldBuilder,
        InputObjectDefBuilder, ObjectDefBuilder, ScalarDefBuilder, Schema, SchemaDefBuilder, Type_,
        UnionDefBuilder,
    };
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_test() {
        let directive = DirectiveBuilder::new("provideTreat")
            .description("Ensures cats get treats.")
            .location("OBJECT")
            .location("FIELD_DEFINITION")
            .location("INPUT_FIELD_DEFINITION")
            .build();
        let schema_def = SchemaDefBuilder::new()
            .query("TryingToFindCatQuery")
            .build();
        let union_def = UnionDefBuilder::new("Pet")
            .description("A union of all animals in a household.")
            .member("Cat")
            .member("Dog")
            .build();
        let object_def = {
            let field_1 = {
                let ty = Type_::named_type("DanglerPoleToys");
                let ty = Type_::list(Box::new(ty));

                FieldBuilder::new("toys", ty)
                    .deprecated("Cats are too spoiled")
                    .build()
            };
            let field_2 = {
                let ty = Type_::named_type("FoodType");

                FieldBuilder::new("food", ty)
                    .description("Dry or wet food?")
                    .build()
            };
            let field_3 = {
                let field = Type_::named_type("Boolean");

                FieldBuilder::new("catGrass", field).build()
            };

            ObjectDefBuilder::new("PetStoreTrip")
                .field(field_1)
                .field(field_2)
                .field(field_3)
                .interface("ShoppingTrip")
                .build()
        };
        let enum_def = {
            let enum_value_1 = EnumValueBuilder::new("CAT_TREE")
                .description("Top bunk of a cat tree.")
                .build();
            let enum_value_2 = EnumValueBuilder::new("BED").build();
            let enum_value_3 = EnumValueBuilder::new("CARDBOARD_BOX")
                .deprecated("Box was recycled.")
                .build();

            EnumDefBuilder::new("NapSpots")
                .description("Favourite cat nap spots.")
                .value(enum_value_1)
                .value(enum_value_2)
                .value(enum_value_3)
                .build()
        };
        let scalar = ScalarDefBuilder::new("NumberOfTreatsPerDay")
            .description("Int representing number of treats received.")
            .build();
        let input_def = {
            let input_field = {
                let ty = Type_::named_type("DanglerPoleToys");
                let ty = Type_::list(Box::new(ty));

                InputFieldBuilder::new("toys", ty)
                    .default("\"Cat Dangler Pole Bird\"")
                    .build()
            };
            let input_value_2 = {
                let ty = Type_::named_type("FavouriteSpots");

                InputFieldBuilder::new("playSpot", ty)
                    .description("Best playime spots, e.g. \"tree\", \"bed\".")
                    .build()
            };

            InputObjectDefBuilder::new("PlayTime")
                .field(input_field)
                .field(input_value_2)
                .build()
        };

        let schema = Schema::new()
            .directive(directive)
            .schema(schema_def)
            .union(union_def)
            .object(object_def)
            .enum_(enum_def)
            .scalar(scalar)
            .input(input_def)
            .finish();

        assert_eq!(
            schema,
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

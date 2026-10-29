use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::InputValueDefinition;
use apollo_compiler::ast::Type;
use apollo_compiler::ast::Value;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

// Long strings and lists don't add coverage, and used to consume most of
// the input, starving the definitions generated after them.
const MAX_STRING_VALUE_LEN: usize = 32;
const MAX_LIST_VALUE_LEN: usize = 3;

impl DocumentBuilder<'_> {
    pub fn input_value_for_type(&mut self, ty: &Type) -> ArbitraryResult<Value> {
        self.input_value_at_depth(ty, 0)
    }

    fn input_value_at_depth(&mut self, ty: &Type, depth: usize) -> ArbitraryResult<Value> {
        let gen_val = |doc_builder: &mut DocumentBuilder<'_>| -> ArbitraryResult<Value> {
            if crate::ty::is_builtin(ty) {
                match ty.inner_named_type().as_str() {
                    "String" => Ok(Value::String(
                        doc_builder.limited_string(MAX_STRING_VALUE_LEN)?,
                    )),
                    "Int" => Ok(Value::Int(doc_builder.u.arbitrary::<i32>()?.into())),
                    "Float" => Ok(Value::Float(doc_builder.finite_f64()?.into())),
                    "Boolean" => Ok(Value::Boolean(doc_builder.u.arbitrary()?)),
                    "ID" => Ok(Value::Int(doc_builder.u.arbitrary::<i32>()?.into())),
                    other => {
                        unreachable!("{} is not a builtin", other);
                    }
                }
            } else if let Some(enum_index) = doc_builder
                .enum_type_defs
                .iter()
                .position(|e| e.type_name() == ty.inner_named_type())
            {
                Ok(Value::Enum(
                    doc_builder.arbitrary_value_of_enum(enum_index)?,
                ))
            } else if doc_builder
                .input_object_type_defs
                .iter()
                .any(|io| io.type_name() == ty.inner_named_type())
            {
                doc_builder.input_object_value(ty.inner_named_type(), depth)
            } else if doc_builder
                .scalar_type_defs
                .iter()
                .any(|s| s.type_name() == ty.inner_named_type())
            {
                // Custom scalars accept any literal value; generate an Int to be entropy-efficient
                Ok(Value::Int(doc_builder.u.arbitrary::<i32>()?.into()))
            } else {
                panic!("Type {} is not a valid input type", ty.inner_named_type());
            }
        };

        let val = match ty {
            Type::Named(_) | Type::NonNullNamed(_) => gen_val(self)?,
            Type::List(_) => {
                let nb_elt = self.u.int_in_range(0..=MAX_LIST_VALUE_LEN)?;
                Value::List(
                    (0..nb_elt)
                        .map(|_| gen_val(self).map(Node::new))
                        .collect::<ArbitraryResult<Vec<_>>>()?,
                )
            }
            Type::NonNullList(_) => gen_val(self)?,
        };

        Ok(val)
    }

    /// Build a value for an input object from its base definition and
    /// extensions. A @oneOf input object gets exactly one field.
    ///
    /// Base definitions only reference input types created before them, so
    /// their fields can't form a cycle, but extension fields can. Extensions
    /// only add nullable fields, so dropping them past a small depth keeps
    /// the value valid and finite.
    fn input_object_value(&mut self, name: &Name, depth: usize) -> ArbitraryResult<Value> {
        const MAX_DEPTH_WITH_EXTENSIONS: usize = 3;
        let fields: Vec<InputValueDefinition> = self
            .input_object_type_defs
            .iter()
            .filter(|io| io.type_name() == name)
            .filter(|io| !io.is_extension_definition() || depth < MAX_DEPTH_WITH_EXTENSIONS)
            .flat_map(|io| io.input_fields().iter().map(|field| (**field).clone()))
            .collect();
        let fields = if self.is_one_of_input(name) {
            vec![self.u.choose(&fields)?.clone()]
        } else {
            fields
        };
        Ok(Value::Object(
            fields
                .iter()
                .map(|field_def| {
                    Ok((
                        field_def.name.clone(),
                        Node::new(self.input_value_at_depth(&field_def.ty, depth + 1)?),
                    ))
                })
                .collect::<ArbitraryResult<Vec<_>>>()?,
        ))
    }

    /// Create an arbitrary list of `InputValueDefinition`. The caller passes
    /// `directive_location` so directives applied to each value are
    /// filtered against the right context — `InputFieldDefinition` for
    /// input-object fields, `ArgumentDefinition` for argument lists on
    /// fields and directives.
    pub fn input_values_def(
        &mut self,
        directive_location: DirectiveLocation,
        exclude: &IndexSet<Name>,
        self_name: Option<&Name>,
    ) -> ArbitraryResult<Vec<InputValueDefinition>> {
        let arbitrary_iv_num = self.u.int_in_range(2..=5usize)?;
        let mut input_values = Vec::with_capacity(arbitrary_iv_num - 1);
        let mut available_types = self.list_existing_input_types();
        if let Some(owner) = self_name {
            available_types
                .retain(|ty| !self.input_field_type_would_cycle(owner, ty.inner_named_type()));
        }

        for i in 0..arbitrary_iv_num {
            let description = self.optional_description()?;
            let name = self.name_with_index(i)?;
            let mut ty = self.choose_ty(&available_types)?;
            // Prevent required self-referential input object fields, which
            // would make the type impossible to construct.
            if self_name.is_some_and(|n| ty.inner_named_type() == n) {
                ty = ty.nullable();
            }
            let directives = self.directives(directive_location)?;
            let default_value = self
                .u
                .arbitrary()
                .unwrap_or(false)
                .then(|| self.input_value_for_type(&ty))
                .transpose()?;

            if !exclude.contains(&name) {
                input_values.push(InputValueDefinition {
                    description,
                    name,
                    ty: Node::new(ty),
                    default_value: default_value.map(Node::new),
                    directives,
                });
            }
        }

        Ok(input_values)
    }
    /// Create an arbitrary `InputValueDefinition`. The caller passes
    /// `directive_location` for the same reason as
    /// [`input_values_def`](Self::input_values_def).
    pub fn input_value_def(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<InputValueDefinition> {
        let description = self.optional_description()?;
        let name = self.name()?;
        let ty = self.choose_ty(&self.list_existing_input_types())?;
        let directives = self.directives(directive_location)?;
        let default_value = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.input_value_for_type(&ty))
            .transpose()?;

        Ok(InputValueDefinition {
            description,
            name,
            ty: Node::new(ty),
            default_value: default_value.map(Node::new),
            directives,
        })
    }

    fn finite_f64(&mut self) -> arbitrary::Result<f64> {
        loop {
            let val: f64 = self.u.arbitrary()?;
            if val.is_finite() {
                return Ok(val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::generated_against_any;
    use crate::test_support::input;
    use crate::test_support::prop_assert_valid;
    use apollo_compiler::ty;
    use arbitrary::Unstructured;
    use proptest::prelude::*;

    fn longest_string_and_list(value: &Value) -> (usize, usize) {
        match value {
            Value::String(s) => (s.len(), 0),
            Value::List(items) => items
                .iter()
                .map(|value| longest_string_and_list(value))
                .fold((0, items.len()), |(s, l), (si, li)| (s.max(si), l.max(li))),
            _ => (0, 0),
        }
    }

    // Values nest lists inside input objects inside lists.
    const NESTED: &str = "schema { query: Query }
        type Query { f(outer: [Outer]): Int }
        input Outer { first: [Inner] }
        input Inner { value: String }";

    // Values need exactly one field for `One`, and the field added by the
    // extension for `Extended`.
    const ONE_OF_AND_EXTENDED: &str = "schema { query: Query }
        type Query { f(one: One, extended: Extended): Int }
        input One @oneOf { a: Int, b: String }
        input Extended { a: Int }
        extend input Extended { b: String! }";

    proptest! {
        #[test]
        fn operation_values_respect_input_object_definitions(
            case in generated_against_any(
                &[NESTED, ONE_OF_AND_EXTENDED],
                |b| b.operation_definition(),
            ),
        ) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }

    proptest! {
        #[test]
        fn string_and_list_values_stay_short(data in input()) {
            let mut u = Unstructured::new(&data);
            let mut builder = DocumentBuilder::new(&mut u);
            let string = ty!(String);
            let ty = Type::List(Box::new(Type::List(Box::new(string))));
            let value = builder.input_value_for_type(&ty).expect("value for a builtin type");
            let (string_len, list_len) = longest_string_and_list(&value);
            prop_assert!(string_len <= MAX_STRING_VALUE_LEN, "{value:?}");
            prop_assert!(list_len <= MAX_LIST_VALUE_LEN, "{value:?}");
        }
    }
}

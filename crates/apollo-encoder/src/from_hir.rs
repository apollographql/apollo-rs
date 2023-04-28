use crate::*;
use apollo_compiler::hir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FromHirError {
    #[error("float conversion error")]
    FloatCoercionError,
}

impl TryFrom<&hir::ObjectTypeDefinition> for ObjectDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::ObjectTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = ObjectDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for interface in value.implements_interfaces() {
            def.interface(interface.interface().to_owned());
        }

        for field in value.self_fields() {
            def.field(field.try_into()?);
        }

        for directive in value.self_directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::ObjectTypeExtension> for ObjectDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::ObjectTypeExtension) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = ObjectDefinition::new(name);
        def.extend();

        for interface in value.implements_interfaces() {
            def.interface(interface.interface().to_owned());
        }

        for field in value.fields() {
            def.field(field.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InterfaceTypeDefinition> for InterfaceDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InterfaceTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = InterfaceDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for interface in value.implements_interfaces() {
            def.interface(interface.interface().to_owned());
        }

        for field in value.self_fields() {
            def.field(field.try_into()?);
        }

        for directive in value.self_directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InterfaceTypeExtension> for InterfaceDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InterfaceTypeExtension) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = InterfaceDefinition::new(name);
        def.extend();

        for interface in value.implements_interfaces() {
            def.interface(interface.interface().to_owned());
        }

        for field in value.fields() {
            def.field(field.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::ScalarTypeDefinition> for ScalarDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::ScalarTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = ScalarDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.self_directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::ScalarTypeExtension> for ScalarDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::ScalarTypeExtension) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = ScalarDefinition::new(name);
        def.extend();

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::UnionTypeDefinition> for UnionDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::UnionTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = UnionDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for member in value.members() {
            def.member(member.name().to_owned());
        }

        for directive in value.self_directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::UnionTypeExtension> for UnionDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::UnionTypeExtension) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = UnionDefinition::new(name);
        def.extend();

        for member in value.members() {
            def.member(member.name().to_owned());
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::EnumTypeDefinition> for EnumDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::EnumTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = EnumDefinition::new(name);

        for value in value.self_values() {
            def.value(value.try_into()?);
        }

        for directive in value.self_directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::EnumTypeExtension> for EnumDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::EnumTypeExtension) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = EnumDefinition::new(name);
        def.extend();

        for value in value.values() {
            def.value(value.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::EnumValueDefinition> for EnumValue {
    type Error = FromHirError;

    fn try_from(value: &hir::EnumValueDefinition) -> Result<Self, Self::Error> {
        let enum_value = value.enum_value().to_owned();
        let mut def = EnumValue::new(enum_value);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InputObjectTypeDefinition> for InputObjectDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InputObjectTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = InputObjectDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.self_directives() {
            def.directive(directive.try_into()?);
        }

        for input_field in value.self_fields() {
            def.field(input_field.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InputObjectTypeExtension> for InputObjectDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InputObjectTypeExtension) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = InputObjectDefinition::new(name);
        def.extend();

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        for input_field in value.fields() {
            def.field(input_field.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InputValueDefinition> for InputField {
    type Error = FromHirError;

    fn try_from(value: &hir::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let typ = value.ty().try_into()?;
        let mut def = InputField::new(name, typ);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        if let Some(default_value) = value.default_value() {
            let encoder_value: Value = default_value.try_into()?;
            let value_str = format!("{}", encoder_value); //TODO verify this
            def.default_value(value_str);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::FieldDefinition> for FieldDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::FieldDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let field_type = value.ty().try_into()?;
        let mut def = FieldDefinition::new(name, field_type);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for iv_def in value.arguments().input_values() {
            def.arg(iv_def.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::Type> for Type_ {
    type Error = FromHirError;

    fn try_from(value: &hir::Type) -> Result<Self, Self::Error> {
        let ty = match value {
            hir::Type::NonNull { ty: hir_ty, .. } => Type_::NonNull {
                ty: Box::new(hir_ty.as_ref().try_into()?),
            },
            hir::Type::List { ty: hir_ty, .. } => Type_::List {
                ty: Box::new(hir_ty.as_ref().try_into()?),
            },
            hir::Type::Named { name, .. } => Type_::NamedType {
                name: name.to_owned(),
            },
        };

        Ok(ty)
    }
}

impl TryFrom<&hir::Directive> for Directive {
    type Error = FromHirError;

    fn try_from(value: &hir::Directive) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut directive = Directive::new(name);

        for arg in value.arguments() {
            directive.arg(arg.try_into()?);
        }

        Ok(directive)
    }
}

impl TryFrom<&hir::Argument> for Argument {
    type Error = FromHirError;

    fn try_from(value: &hir::Argument) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let value = value.value().try_into()?;
        let arg = Argument::new(name, value);

        Ok(arg)
    }
}

impl TryFrom<&hir::Value> for Value {
    type Error = FromHirError;

    fn try_from(value: &hir::Value) -> Result<Self, Self::Error> {
        let value = match value {
            hir::Value::Variable(v) => Value::Variable(v.name().to_owned()),

            //TODO look more closely at int conversion
            hir::Value::Int(i) => {
                Value::Int(i.to_i32_checked().ok_or(FromHirError::FloatCoercionError)?)
            }
            hir::Value::Float(f) => Value::Float(f.get()),
            hir::Value::String(s) => Value::String(s.clone()),
            hir::Value::Boolean(b) => Value::Boolean(*b),
            hir::Value::Null => Value::Null,
            hir::Value::Enum(e) => Value::Enum(e.src().to_owned()),
            hir::Value::List(l) => Value::List(
                l.iter()
                    .map(TryInto::<Value>::try_into)
                    .collect::<Result<Vec<_>, FromHirError>>()?,
            ),
            hir::Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(n, v)| v.try_into().map(|v: Value| (n.src().to_owned(), v)))
                    .collect::<Result<Vec<_>, FromHirError>>()?,
            ),
        };

        Ok(value)
    }
}

impl TryFrom<&hir::InputValueDefinition> for InputValueDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let iv_type = value.ty().try_into()?;
        let mut def = InputValueDefinition::new(name, iv_type);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        if let Some(default_value) = value.default_value() {
            let encoder_value: Value = default_value.try_into()?;
            let value_str = format!("{}", encoder_value); //TODO verify this
            def.default_value(value_str);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::DirectiveDefinition> for DirectiveDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::DirectiveDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = DirectiveDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        if value.repeatable() {
            def.repeatable();
        }

        for arg in value.arguments().input_values() {
            def.arg(arg.try_into()?);
        }

        for directive_loc in value.directive_locations() {
            def.location(directive_loc.name().to_owned());
        }

        Ok(def)
    }
}

impl TryFrom<&hir::FragmentDefinition> for FragmentDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::FragmentDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let type_cond = value.type_condition().to_owned();
        let selection_set = value.selection_set().try_into()?;

        let def = FragmentDefinition::new(name, TypeCondition::new(type_cond), selection_set);

        Ok(def)
    }
}

impl TryFrom<&hir::SelectionSet> for SelectionSet {
    type Error = FromHirError;

    fn try_from(value: &hir::SelectionSet) -> Result<Self, Self::Error> {
        let mut selection_set = SelectionSet::new();

        for selection in value.selection() {
            selection_set.selection(selection.try_into()?)
        }

        Ok(selection_set)
    }
}

impl TryFrom<&hir::Selection> for Selection {
    type Error = FromHirError;

    fn try_from(value: &hir::Selection) -> Result<Self, Self::Error> {
        let selection = match value {
            hir::Selection::Field(field) => Selection::Field(field.as_ref().try_into()?),
            hir::Selection::FragmentSpread(fragment) => {
                Selection::FragmentSpread(fragment.as_ref().try_into()?)
            }
            hir::Selection::InlineFragment(fragment) => {
                Selection::InlineFragment(fragment.as_ref().try_into()?)
            }
        };

        Ok(selection)
    }
}

impl TryFrom<&hir::Field> for Field {
    type Error = FromHirError;

    fn try_from(value: &hir::Field) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut field = Field::new(name);

        field.alias(value.alias().map(|a| a.0.clone()));

        for arg in value.arguments() {
            field.argument(arg.try_into()?);
        }

        for directive in value.directives() {
            field.directive(directive.try_into()?);
        }

        if !value.selection_set().selection().is_empty() {
            field.selection_set(Some(value.selection_set().try_into()?));
        }

        Ok(field)
    }
}

impl TryFrom<&hir::FragmentSpread> for FragmentSpread {
    type Error = FromHirError;

    fn try_from(value: &hir::FragmentSpread) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut fragment = FragmentSpread::new(name);

        for directive in value.directives() {
            fragment.directive(directive.try_into()?);
        }

        Ok(fragment)
    }
}

impl TryFrom<&hir::InlineFragment> for InlineFragment {
    type Error = FromHirError;

    fn try_from(value: &hir::InlineFragment) -> Result<Self, Self::Error> {
        let selection_set = value.selection_set().try_into()?;
        let mut fragment = InlineFragment::new(selection_set);

        fragment.type_condition(
            value
                .type_condition()
                .map(|tc| TypeCondition::new(tc.to_owned())),
        );

        for directive in value.directives() {
            fragment.directive(directive.try_into()?);
        }

        Ok(fragment)
    }
}

impl TryFrom<&hir::VariableDefinition> for VariableDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::VariableDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let ty = value.ty().try_into()?;
        let mut def = VariableDefinition::new(name, ty);

        if let Some(default_value) = value.default_value() {
            def.default_value(default_value.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::OperationDefinition> for OperationDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::OperationDefinition) -> Result<Self, Self::Error> {
        let operation_type = value.operation_ty().try_into()?;
        let selection_set = value.selection_set().try_into()?;

        let mut def = OperationDefinition::new(operation_type, selection_set);

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        for var in value.variables() {
            def.variable_definition(var.try_into()?);
        }

        Ok(def)
    }
}

impl TryInto<OperationType> for hir::OperationType {
    type Error = FromHirError;

    fn try_into(self) -> Result<OperationType, Self::Error> {
        Ok(match self {
            hir::OperationType::Query => OperationType::Query,
            hir::OperationType::Mutation => OperationType::Mutation,
            hir::OperationType::Subscription => OperationType::Subscription,
        })
    }
}

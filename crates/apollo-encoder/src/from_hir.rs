use crate::{self as encoder};
use apollo_compiler::hir;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FromHirError {
    #[error("float conversion error")]
    FloatCoercionError,
}

impl TryFrom<&hir::ObjectTypeDefinition> for encoder::ObjectDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::ObjectTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::ObjectDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for interface in value.implements_interfaces() {
            def.interface(interface.interface().to_owned());
        }

        for field in value.fields_definition() {
            def.field(field.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InterfaceTypeDefinition> for encoder::InterfaceDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InterfaceTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::InterfaceDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for interface in value.implements_interfaces() {
            def.interface(interface.interface().to_owned());
        }

        for field in value.fields_definition() {
            def.field(field.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::ScalarTypeDefinition> for encoder::ScalarDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::ScalarTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::ScalarDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::UnionTypeDefinition> for encoder::UnionDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::UnionTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::UnionDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for member in value.union_members() {
            def.member(member.name().to_owned());
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::EnumTypeDefinition> for encoder::EnumDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::EnumTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::EnumDefinition::new(name);

        for value in value.enum_values_definition() {
            def.value(value.try_into()?);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::EnumValueDefinition> for encoder::EnumValue {
    type Error = FromHirError;

    fn try_from(value: &hir::EnumValueDefinition) -> Result<Self, Self::Error> {
        let enum_value = value.enum_value().to_owned();
        let mut def = encoder::EnumValue::new(enum_value);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InputObjectTypeDefinition> for encoder::InputObjectDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InputObjectTypeDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::InputObjectDefinition::new(name);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        for input_field in value.input_fields_definition() {
            def.field(input_field.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::InputValueDefinition> for encoder::InputField {
    type Error = FromHirError;

    fn try_from(value: &hir::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let typ = value.ty().try_into()?;
        let mut def = encoder::InputField::new(name, typ);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        if let Some(default_value) = value.default_value() {
            let encoder_value: encoder::Value = default_value.try_into()?;
            let value_str = format!("{}", encoder_value); //TODO verify this
            def.default_value(value_str);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::FieldDefinition> for encoder::FieldDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::FieldDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let field_type = value.ty().try_into()?;
        let mut def = encoder::FieldDefinition::new(name, field_type);

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

impl TryFrom<&hir::Type> for encoder::Type_ {
    type Error = FromHirError;

    fn try_from(value: &hir::Type) -> Result<Self, Self::Error> {
        let ty = match value {
            hir::Type::NonNull { ty: hir_ty, .. } => encoder::Type_::NonNull {
                ty: Box::new(hir_ty.as_ref().try_into()?),
            },
            hir::Type::List { ty: hir_ty, .. } => encoder::Type_::List {
                ty: Box::new(hir_ty.as_ref().try_into()?),
            },
            hir::Type::Named { name, .. } => encoder::Type_::NamedType {
                name: name.to_owned(),
            },
        };

        Ok(ty)
    }
}

impl TryFrom<&hir::Directive> for encoder::Directive {
    type Error = FromHirError;

    fn try_from(value: &hir::Directive) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut directive = encoder::Directive::new(name);

        for arg in value.arguments() {
            directive.arg(arg.try_into()?);
        }

        Ok(directive)
    }
}

impl TryFrom<&hir::Argument> for encoder::Argument {
    type Error = FromHirError;

    fn try_from(value: &hir::Argument) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let value = value.value().try_into()?;
        let arg = encoder::Argument::new(name, value);

        Ok(arg)
    }
}

impl TryFrom<&hir::Value> for encoder::Value {
    type Error = FromHirError;

    fn try_from(value: &hir::Value) -> Result<Self, Self::Error> {
        let value = match value {
            hir::Value::Variable(v) => encoder::Value::Variable(v.name().to_owned()),

            //TODO look more closely at int conversion
            hir::Value::Int(i) => {
                encoder::Value::Int(i.to_i32_checked().ok_or(FromHirError::FloatCoercionError)?)
            }
            hir::Value::Float(f) => encoder::Value::Float(f.get()),
            hir::Value::String(s) => encoder::Value::String(s.clone()),
            hir::Value::Boolean(b) => encoder::Value::Boolean(*b),
            hir::Value::Null => encoder::Value::Null,
            hir::Value::Enum(e) => encoder::Value::Enum(e.src().to_owned()),
            hir::Value::List(l) => encoder::Value::List(
                l.iter()
                    .map(TryInto::<encoder::Value>::try_into)
                    .collect::<Result<Vec<_>, FromHirError>>()?,
            ),
            hir::Value::Object(fields) => encoder::Value::Object(
                fields
                    .iter()
                    .map(|(n, v)| {
                        v.try_into()
                            .map(|v: encoder::Value| (n.src().to_owned(), v))
                    })
                    .collect::<Result<Vec<_>, FromHirError>>()?,
            ),
        };

        Ok(value)
    }
}

impl TryFrom<&hir::InputValueDefinition> for encoder::InputValueDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::InputValueDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let iv_type = value.ty().try_into()?;
        let mut def = encoder::InputValueDefinition::new(name, iv_type);

        if let Some(description) = value.description().map(str::to_string) {
            def.description(description);
        }

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        if let Some(default_value) = value.default_value() {
            let encoder_value: encoder::Value = default_value.try_into()?;
            let value_str = format!("{}", encoder_value); //TODO verify this
            def.default_value(value_str);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::DirectiveDefinition> for encoder::DirectiveDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::DirectiveDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut def = encoder::DirectiveDefinition::new(name);

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

impl TryFrom<&hir::FragmentDefinition> for encoder::FragmentDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::FragmentDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let type_cond = value.type_condition().to_owned();
        let selection_set = value.selection_set().try_into()?;

        let def = encoder::FragmentDefinition::new(
            name,
            encoder::TypeCondition::new(type_cond),
            selection_set,
        );

        Ok(def)
    }
}

impl TryFrom<&hir::SelectionSet> for encoder::SelectionSet {
    type Error = FromHirError;

    fn try_from(value: &hir::SelectionSet) -> Result<Self, Self::Error> {
        let mut selection_set = encoder::SelectionSet::new();

        for selection in value.selection() {
            selection_set.selection(selection.try_into()?)
        }

        Ok(selection_set)
    }
}

impl TryFrom<&hir::Selection> for encoder::Selection {
    type Error = FromHirError;

    fn try_from(value: &hir::Selection) -> Result<Self, Self::Error> {
        let selection = match value {
            hir::Selection::Field(field) => encoder::Selection::Field(field.as_ref().try_into()?),
            hir::Selection::FragmentSpread(fragment) => {
                encoder::Selection::FragmentSpread(fragment.as_ref().try_into()?)
            }
            hir::Selection::InlineFragment(fragment) => {
                encoder::Selection::InlineFragment(fragment.as_ref().try_into()?)
            }
        };

        Ok(selection)
    }
}

impl TryFrom<&hir::Field> for encoder::Field {
    type Error = FromHirError;

    fn try_from(value: &hir::Field) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut field = encoder::Field::new(name);

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

impl TryFrom<&hir::FragmentSpread> for encoder::FragmentSpread {
    type Error = FromHirError;

    fn try_from(value: &hir::FragmentSpread) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let mut fragment = encoder::FragmentSpread::new(name);

        for directive in value.directives() {
            fragment.directive(directive.try_into()?);
        }

        Ok(fragment)
    }
}

impl TryFrom<&hir::InlineFragment> for encoder::InlineFragment {
    type Error = FromHirError;

    fn try_from(value: &hir::InlineFragment) -> Result<Self, Self::Error> {
        let selection_set = value.selection_set().try_into()?;
        let mut fragment = encoder::InlineFragment::new(selection_set);

        fragment.type_condition(
            value
                .type_condition()
                .map(|tc| encoder::TypeCondition::new(tc.to_owned())),
        );

        for directive in value.directives() {
            fragment.directive(directive.try_into()?);
        }

        Ok(fragment)
    }
}

impl TryFrom<&hir::VariableDefinition> for encoder::VariableDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::VariableDefinition) -> Result<Self, Self::Error> {
        let name = value.name().to_owned();
        let ty = value.ty().try_into()?;
        let mut def = encoder::VariableDefinition::new(name, ty);

        if let Some(default_value) = value.default_value() {
            def.default_value(default_value.try_into()?);
        }

        Ok(def)
    }
}

impl TryFrom<&hir::OperationDefinition> for encoder::OperationDefinition {
    type Error = FromHirError;

    fn try_from(value: &hir::OperationDefinition) -> Result<Self, Self::Error> {
        let operation_type = value.operation_ty().try_into()?;
        let selection_set = value.selection_set().try_into()?;

        let mut def = encoder::OperationDefinition::new(operation_type, selection_set);

        for directive in value.directives() {
            def.directive(directive.try_into()?);
        }

        for var in value.variables() {
            def.variable_definition(var.try_into()?);
        }

        Ok(def)
    }
}

impl TryInto<encoder::OperationType> for hir::OperationType {
    type Error = FromHirError;

    fn try_into(self) -> Result<encoder::OperationType, Self::Error> {
        Ok(match self {
            hir::OperationType::Query => encoder::OperationType::Query,
            hir::OperationType::Mutation => encoder::OperationType::Mutation,
            hir::OperationType::Subscription => encoder::OperationType::Subscription,
        })
    }
}

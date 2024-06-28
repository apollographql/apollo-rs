use crate::ast;
use crate::schema;
use crate::schema::ExtendedType;
use crate::schema::UnionType;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DiagnosticList;
use crate::Node;

pub(crate) fn validate_union_definitions(diagnostics: &mut DiagnosticList, schema: &crate::Schema) {
    for ty in schema.types.values() {
        if let ExtendedType::Union(def) = ty {
            validate_union_definition(diagnostics, schema, def);
        }
    }
}

pub(crate) fn validate_union_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    union_def: &Node<UnionType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        union_def.directives.iter_ast(),
        ast::DirectiveLocation::Union,
        // unions don't use variables
        Default::default(),
    );

    for union_member in &union_def.members {
        let member_location = union_member.location();
        // TODO: (?) A Union type must include one or more unique member types.

        match schema.types.get(&union_member.name) {
            None => {
                // Union member must be defined.
                diagnostics.push(
                    member_location,
                    DiagnosticData::UndefinedDefinition {
                        name: union_member.name.clone(),
                    },
                );
            }
            Some(schema::ExtendedType::Object(_)) => {} // good
            Some(ty) => {
                // Union member must be of object type.
                diagnostics.push(
                    member_location,
                    DiagnosticData::UnionMemberObjectType {
                        name: union_member.name.clone(),
                        describe_type: ty.describe(),
                    },
                );
            }
        }
    }

    // validate there is at least one union member on the union type
    // https://spec.graphql.org/draft/#sel-HAHdfFBABAB6Bw3R
    if union_def.members.is_empty() {
        diagnostics.push(
            union_def.location(),
            DiagnosticData::EmptyMemberSet {
                type_name: union_def.name.clone(),
                type_location: union_def.location(),
                extensions_locations: union_def
                    .extensions()
                    .iter()
                    .map(|ext| ext.location())
                    .collect(),
            },
        );
    }
}

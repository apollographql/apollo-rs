use super::BuildError;
use crate::validation::Details;
use crate::validation::Diagnostics;
use crate::Schema;

pub(crate) fn validate_schema(errors: &mut Diagnostics, schema: &Schema) {
    for (&file_id, source) in &schema.sources {
        source.validate_parse_errors(errors, file_id)
    }
    for build_error in &schema.build_errors {
        validate_build_error(errors, build_error)
    }
    // TODO
}

fn validate_build_error(errors: &mut Diagnostics, build_error: &BuildError) {
    match build_error {
        BuildError::ExecutableDefinition { location, .. }
        | BuildError::SchemaDefinitionCollision { location, .. }
        | BuildError::DirectiveDefinitionCollision { location, .. }
        | BuildError::TypeDefinitionCollision { location, .. }
        | BuildError::BuiltInScalarTypeRedefinition { location, .. }
        | BuildError::OrphanSchemaExtension { location, .. }
        | BuildError::OrphanTypeExtension { location, .. }
        | BuildError::TypeExtensionKindMismatch { location, .. }
        | BuildError::DuplicateRootOperation { location, .. }
        | BuildError::DuplicateImplementsInterfaceInObject { location, .. }
        | BuildError::DuplicateImplementsInterfaceInInterface { location, .. }
        | BuildError::ObjectFieldNameCollision { location, .. }
        | BuildError::InterfaceFieldNameCollision { location, .. }
        | BuildError::EnumValueNameCollision { location, .. }
        | BuildError::UnionMemberNameCollision { location, .. }
        | BuildError::InputFieldNameCollision { location, .. } => {
            errors.push(*location, Details::SchemaBuildError(build_error.clone()))
        }
    }
}

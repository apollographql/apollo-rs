//! This is a generated file, please do not edit.

#![allow(bad_style, missing_docs, unreachable_pub)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    BANG,
    L_PAREN,
    R_PAREN,
    L_CURLY,
    R_CURLY,
    L_BRACK,
    R_BRACK,
    AT,
    DOLLAR,
    PIPE,
    SPREAD,
    EQ,
    COLON,
    query_KW,
    mutation_KW,
    subscription_KW,
    fragment_KW,
    on_KW,
    null_KW,
    extend_KW,
    schema_KW,
    scalar_KW,
    implements_KW,
    interface_KW,
    union_KW,
    enum_KW,
    input_KW,
    directive_KW,
    QUERY_KW,
    MUTATION_KW,
    SUBSCRIPTION_KW,
    FIELD_KW,
    FRAGMENT_DEFINITION_KW,
    FRAGMENT_SPREAD_KW,
    INLINE_FRAGMENT_KW,
    SCHEMA_KW,
    OBJECT_KW,
    FIELD_DEFINITION_KW,
    ARGUMENT_DEFINITION_KW,
    INTERFACE_KW,
    UNION_KW,
    ENUM_KW,
    ENUM_VALUE_KW,
    INPUT_OBJECT_KW,
    INPUT_FIELD_DEFINITION_KW,
    INT_NUMBER,
    STRING,
    RAW_STRING,
    IDENT,
    WHITESPACE,
    COMMENT,
    NAME,
    INT_VALUE,
    INTEGER_PART,
    NEGATIVE_SIGN,
    NON_ZERO_DIGIT,
    DIGIT,
    FLOAT_VALUE,
    FRACTIONAL_PART,
    EXPONENT_PART,
    EXPONENT_INDICATOR,
    SIGN,
    STRING_VALUE,
    STRING_CHARACTER,
    BLOACK_STRING_CHARACTER,
    DOCUMENT,
    DEFINITION,
    EXECUTABLE_DEFINITION,
    TYPE_SYSTEM_DEFINITION,
    TYPE_SYSTEM_EXTENSION,
    OPERATION_DEFINITION,
    FRAGMENT_DEFINITION,
    OPERATION_TYPE,
    VARIABLE_DEFINITIONS,
    DIRECTIVES,
    SELECTION_SET,
    SELECTION,
    FIELD,
    FRAGMENT_SPREAD,
    INLINE_FRAGMENT,
    ALIAS,
    ARGUMENTS,
    ARGUMENT,
    VALUE,
    FRAGMENT_NAME,
    INLINE_FRAGMENT,
    TYPE_CONDITION,
    FRAGMENT_DEFINITION,
    FRAGMENT_NAME,
    VARIABLE,
    BOOLEAN_VALUE,
    NULL_VALUE,
    ENUM_VALUE,
    LIST_VALUE,
    OBJECT_VALUE,
    OBJECT_FIELD,
    VARIABLE_DEFINITIONS,
    VARIABLE_DEFINITION,
    TYPE,
    DEFAULT_VALUE,
    NAMED_TYPE,
    LIST_TYPE,
    NON_NULL_TYPE,
    SCHEMA_DEFINITION,
    TYPE_DEFINITION,
    DIRECTIVE_DEFINITION,
    SCHEMA_EXTENSION,
    TYPE_EXTENSION,
    OPERATION_TYPE_DEFINITION,
    OPERATION_TYPE,
    DESCRIPTION,
    SCALAR_TYPE_DEFINITION,
    OBJECT_TYPE_DEFINITION,
    INTERFACE_TYPE_DEFINITION,
    UNION_TYPE_DEFINITION,
    ENUM_TYPE_DEFINITION,
    INPUT_OBJECT_TYPE_DEFINITION,
    SCALAR_TYPE_EXTENSION,
    OBJECT_TYPE_EXTENSION,
    INTERFACE_TYPE_EXTENSION,
    UNION_TYPE_EXTENSION,
    ENUM_TYPE_EXTENSION,
    INPUT_OBJECT_TYPEEXTENSION,
    IMPLEMENTS_INTERFACES,
    FIELDS_DEFINITION,
    FIELD_DEFINITION,
    ARGUMENTS_DEFINITION,
    INPUT_VALUE_DEFINITION,
    UNION_MEMBER_TYPES,
    ENUM_VALUES_DEFINITION,
    ENUM_VALUE_DEFINITION,
    INPUT_FIELDS_DEFINITION,
    INPUT_VALUE_DEFINITION,
    DIRECTIVE_LOCATIONS,
    DIRECTIVE_LOCATION,
    EXECUTABLE_DIRECTIVE_LOCATION,
    TYPE_SYSTEM_DIRECTIVE_LOCATION,
    #[doc(hidden)]
    __LAST,
}
use self::SyntaxKind::*;
impl SyntaxKind {
    pub fn is_keyword(self) -> bool {
        match self {
            query_KW
            | mutation_KW
            | subscription_KW
            | fragment_KW
            | on_KW
            | null_KW
            | extend_KW
            | schema_KW
            | scalar_KW
            | implements_KW
            | interface_KW
            | union_KW
            | enum_KW
            | input_KW
            | directive_KW
            | QUERY_KW
            | MUTATION_KW
            | SUBSCRIPTION_KW
            | FIELD_KW
            | FRAGMENT_DEFINITION_KW
            | FRAGMENT_SPREAD_KW
            | INLINE_FRAGMENT_KW
            | SCHEMA_KW
            | OBJECT_KW
            | FIELD_DEFINITION_KW
            | ARGUMENT_DEFINITION_KW
            | INTERFACE_KW
            | UNION_KW
            | ENUM_KW
            | ENUM_VALUE_KW
            | INPUT_OBJECT_KW
            | INPUT_FIELD_DEFINITION_KW => true,
            _ => false,
        }
    }
    pub fn is_punct(self) -> bool {
        match self {
            BANG | L_PAREN | R_PAREN | L_CURLY | R_CURLY | L_BRACK | R_BRACK | AT | DOLLAR
            | PIPE | SPREAD | EQ | COLON => true,
            _ => false,
        }
    }
    pub fn is_literal(self) -> bool {
        match self {
            INT_NUMBER | STRING | RAW_STRING => true,
            _ => false,
        }
    }
    pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
        let kw = match ident {
            "query" => query_KW,
            "mutation" => mutation_KW,
            "subscription" => subscription_KW,
            "fragment" => fragment_KW,
            "on" => on_KW,
            "null" => null_KW,
            "extend" => extend_KW,
            "schema" => schema_KW,
            "scalar" => scalar_KW,
            "implements" => implements_KW,
            "interface" => interface_KW,
            "union" => union_KW,
            "enum" => enum_KW,
            "input" => input_KW,
            "directive" => directive_KW,
            "QUERY" => QUERY_KW,
            "MUTATION" => MUTATION_KW,
            "SUBSCRIPTION" => SUBSCRIPTION_KW,
            "FIELD" => FIELD_KW,
            "FRAGMENT_DEFINITION" => FRAGMENT_DEFINITION_KW,
            "FRAGMENT_SPREAD" => FRAGMENT_SPREAD_KW,
            "INLINE_FRAGMENT" => INLINE_FRAGMENT_KW,
            "SCHEMA" => SCHEMA_KW,
            "OBJECT" => OBJECT_KW,
            "FIELD_DEFINITION" => FIELD_DEFINITION_KW,
            "ARGUMENT_DEFINITION" => ARGUMENT_DEFINITION_KW,
            "INTERFACE" => INTERFACE_KW,
            "UNION" => UNION_KW,
            "ENUM" => ENUM_KW,
            "ENUM_VALUE" => ENUM_VALUE_KW,
            "INPUT_OBJECT" => INPUT_OBJECT_KW,
            "INPUT_FIELD_DEFINITION" => INPUT_FIELD_DEFINITION_KW,
            _ => return None,
        };
        Some(kw)
    }
    pub fn from_char(c: char) -> Option<SyntaxKind> {
        let tok = match c {
            ')' => BANG,
            '(' => L_PAREN,
            ')' => R_PAREN,
            '{' => L_CURLY,
            '}' => R_CURLY,
            '[' => L_BRACK,
            ']' => R_BRACK,
            '@' => AT,
            '$' => DOLLAR,
            '|' => PIPE,
            '=' => EQ,
            ':' => COLON,
            _ => return None,
        };
        Some(tok)
    }
}
#[macro_export]
macro_rules ! T { [')'] => { $ crate :: SyntaxKind :: BANG } ; ['('] => { $ crate :: SyntaxKind :: L_PAREN } ; [')'] => { $ crate :: SyntaxKind :: R_PAREN } ; ['{'] => { $ crate :: SyntaxKind :: L_CURLY } ; ['}'] => { $ crate :: SyntaxKind :: R_CURLY } ; ['['] => { $ crate :: SyntaxKind :: L_BRACK } ; [']'] => { $ crate :: SyntaxKind :: R_BRACK } ; [@] => { $ crate :: SyntaxKind :: AT } ; [$] => { $ crate :: SyntaxKind :: DOLLAR } ; [|] => { $ crate :: SyntaxKind :: PIPE } ; [...] => { $ crate :: SyntaxKind :: SPREAD } ; [=] => { $ crate :: SyntaxKind :: EQ } ; [:] => { $ crate :: SyntaxKind :: COLON } ; [query] => { $ crate :: SyntaxKind :: query_KW } ; [mutation] => { $ crate :: SyntaxKind :: mutation_KW } ; [subscription] => { $ crate :: SyntaxKind :: subscription_KW } ; [fragment] => { $ crate :: SyntaxKind :: fragment_KW } ; [on] => { $ crate :: SyntaxKind :: on_KW } ; [null] => { $ crate :: SyntaxKind :: null_KW } ; [extend] => { $ crate :: SyntaxKind :: extend_KW } ; [schema] => { $ crate :: SyntaxKind :: schema_KW } ; [scalar] => { $ crate :: SyntaxKind :: scalar_KW } ; [implements] => { $ crate :: SyntaxKind :: implements_KW } ; [interface] => { $ crate :: SyntaxKind :: interface_KW } ; [union] => { $ crate :: SyntaxKind :: union_KW } ; [enum] => { $ crate :: SyntaxKind :: enum_KW } ; [input] => { $ crate :: SyntaxKind :: input_KW } ; [directive] => { $ crate :: SyntaxKind :: directive_KW } ; [QUERY] => { $ crate :: SyntaxKind :: QUERY_KW } ; [MUTATION] => { $ crate :: SyntaxKind :: MUTATION_KW } ; [SUBSCRIPTION] => { $ crate :: SyntaxKind :: SUBSCRIPTION_KW } ; [FIELD] => { $ crate :: SyntaxKind :: FIELD_KW } ; [FRAGMENT_DEFINITION] => { $ crate :: SyntaxKind :: FRAGMENT_DEFINITION_KW } ; [FRAGMENT_SPREAD] => { $ crate :: SyntaxKind :: FRAGMENT_SPREAD_KW } ; [INLINE_FRAGMENT] => { $ crate :: SyntaxKind :: INLINE_FRAGMENT_KW } ; [SCHEMA] => { $ crate :: SyntaxKind :: SCHEMA_KW } ; [OBJECT] => { $ crate :: SyntaxKind :: OBJECT_KW } ; [FIELD_DEFINITION] => { $ crate :: SyntaxKind :: FIELD_DEFINITION_KW } ; [ARGUMENT_DEFINITION] => { $ crate :: SyntaxKind :: ARGUMENT_DEFINITION_KW } ; [INTERFACE] => { $ crate :: SyntaxKind :: INTERFACE_KW } ; [UNION] => { $ crate :: SyntaxKind :: UNION_KW } ; [ENUM] => { $ crate :: SyntaxKind :: ENUM_KW } ; [ENUM_VALUE] => { $ crate :: SyntaxKind :: ENUM_VALUE_KW } ; [INPUT_OBJECT] => { $ crate :: SyntaxKind :: INPUT_OBJECT_KW } ; [INPUT_FIELD_DEFINITION] => { $ crate :: SyntaxKind :: INPUT_FIELD_DEFINITION_KW } ; [lifetime_ident] => { $ crate :: SyntaxKind :: LIFETIME_IDENT } ; [ident] => { $ crate :: SyntaxKind :: IDENT } ; [shebang] => { $ crate :: SyntaxKind :: SHEBANG } ; }

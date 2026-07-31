use apollo_compiler::schema::SchemaBuilder;
use apollo_compiler::Schema;
use expect_test::expect;

#[test]
fn handles_built_in_scalar_redefinition() {
    let schema = r#"
  scalar String
  scalar Int
  scalar Float
  scalar Boolean
  scalar ID

  type Query {
    foo: String
  }
  "#;

    let errors = Schema::parse_and_validate(schema, "schema.graphql")
        .expect_err("should be invalid schema")
        .errors;
    let expected = expect![[r#"
        Error: built-in scalar definitions must be omitted
           ╭─[ schema.graphql:2:3 ]
           │
         2 │   scalar String
           │   ──────┬──────  
           │         ╰──────── remove this scalar definition
        ───╯
        Error: built-in scalar definitions must be omitted
           ╭─[ schema.graphql:3:3 ]
           │
         3 │   scalar Int
           │   ─────┬────  
           │        ╰────── remove this scalar definition
        ───╯
        Error: built-in scalar definitions must be omitted
           ╭─[ schema.graphql:4:3 ]
           │
         4 │   scalar Float
           │   ──────┬─────  
           │         ╰─────── remove this scalar definition
        ───╯
        Error: built-in scalar definitions must be omitted
           ╭─[ schema.graphql:5:3 ]
           │
         5 │   scalar Boolean
           │   ───────┬──────  
           │          ╰──────── remove this scalar definition
        ───╯
        Error: built-in scalar definitions must be omitted
           ╭─[ schema.graphql:6:3 ]
           │
         6 │   scalar ID
           │   ────┬────  
           │       ╰────── remove this scalar definition
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());

    let builder = SchemaBuilder::new().ignore_builtin_redefinitions();
    let _ = builder
        .parse(schema, "schema.graphql")
        .build()
        .expect("schema parsed successfully");
}

#[test]
fn handles_built_in_type_redefinition() {
    let schema = r#"
     type __Directive {
       name: String!
       description: String!
       isRepeatable: String!
       args: __InputValue
       locations: String!
     }

     type __Schema {
       description: String
       types: [__Type!]
       queryType: __Type!
       mutationType: __Type
       subscriptionType: __Type
       directives: [__Directive!]
     }

     type __Type {
       kind: __TypeKind!
       name: String
       description: String
       fields: [__Field!]
       interfaces: [__Type!]
       possibleTypes: [__Type!]
       enumValues: [__EnumValue!]
       inputFields: [__InputValue!]
       ofType: __Type
       specifiedByURL: String
     }
     
     enum __TypeKind {
       SCALAR
       OBJECT
       INTERFACE
       UNION
       ENUM
       INPUT_OBJECT
       LIST
     }
     
     type __Field {
       name: String!
       description: String
       args: [__InputValue!]!
       type: __Type!
       isDeprecated: Boolean!
       deprecationReason: String
     }
     
     type __InputValue {
       name: String!
       description: String
       type: __Type!
       defaultValue: String
       deprecationReason: String
     }
     
     type __EnumValue {
       name: String!
       description: String
       deprecationReason: String
     }
    
     type Query {
       foo: String
     }
     "#;

    let errors = Schema::parse_and_validate(schema, "schema.graphql")
        .expect_err("should be invalid schema")
        .errors;
    let expected = expect![[r#"
        Error: the type `__Directive` is defined multiple times in the schema
            ╭─[ built_in.graphql:87:6 ]
            │
         87 │ type __Directive {
            │      ─────┬─────  
            │           ╰─────── previous definition of `__Directive` here
            │
            ├─[ schema.graphql:2:11 ]
            │
          2 │      type __Directive {
            │           ─────┬─────  
            │                ╰─────── `__Directive` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
        Error: the type `__Schema` is defined multiple times in the schema
            ╭─[ built_in.graphql:2:6 ]
            │
          2 │ type __Schema {
            │      ────┬───  
            │          ╰───── previous definition of `__Schema` here
            │
            ├─[ schema.graphql:10:11 ]
            │
         10 │      type __Schema {
            │           ────┬───  
            │               ╰───── `__Schema` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
        Error: the type `__Type` is defined multiple times in the schema
            ╭─[ built_in.graphql:17:6 ]
            │
         17 │ type __Type {
            │      ───┬──  
            │         ╰──── previous definition of `__Type` here
            │
            ├─[ schema.graphql:19:11 ]
            │
         19 │      type __Type {
            │           ───┬──  
            │              ╰──── `__Type` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
        Error: the type `__TypeKind` is defined multiple times in the schema
            ╭─[ built_in.graphql:38:6 ]
            │
         38 │ enum __TypeKind {
            │      ─────┬────  
            │           ╰────── previous definition of `__TypeKind` here
            │
            ├─[ schema.graphql:32:11 ]
            │
         32 │      enum __TypeKind {
            │           ─────┬────  
            │                ╰────── `__TypeKind` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
        Error: the type `__Field` is defined multiple times in the schema
            ╭─[ built_in.graphql:58:6 ]
            │
         58 │ type __Field {
            │      ───┬───  
            │         ╰───── previous definition of `__Field` here
            │
            ├─[ schema.graphql:42:11 ]
            │
         42 │      type __Field {
            │           ───┬───  
            │              ╰───── `__Field` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
        Error: the type `__InputValue` is defined multiple times in the schema
            ╭─[ built_in.graphql:68:6 ]
            │
         68 │ type __InputValue {
            │      ──────┬─────  
            │            ╰─────── previous definition of `__InputValue` here
            │
            ├─[ schema.graphql:51:11 ]
            │
         51 │      type __InputValue {
            │           ──────┬─────  
            │                 ╰─────── `__InputValue` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
        Error: the type `__EnumValue` is defined multiple times in the schema
            ╭─[ built_in.graphql:79:6 ]
            │
         79 │ type __EnumValue {
            │      ─────┬─────  
            │           ╰─────── previous definition of `__EnumValue` here
            │
            ├─[ schema.graphql:59:11 ]
            │
         59 │      type __EnumValue {
            │           ─────┬─────  
            │                ╰─────── `__EnumValue` redefined here
            │ 
            │ Help: remove or rename one of the definitions, or use `extend`
        ────╯
    "#]];
    expected.assert_eq(&errors.to_string());

    let builder = SchemaBuilder::new().ignore_builtin_redefinitions();
    let _ = builder
        .parse(schema, "schema.graphql")
        .build()
        .expect("schema parsed successfully");
}

use apollo_compiler::ast;
use apollo_compiler::ast::Value;
use apollo_compiler::name;
use apollo_compiler::parse_mixed_validate;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::Node;

#[test]
fn it_raises_undefined_variable_in_query_error() {
    let input = r#"
query ExampleQuery {
  topProducts(first: $undefinedVariable) {
    name
  }

  me {
    ... on User {
      id
      name
      profilePic(size: $dimensions)
      status
    }
  }
}

type Query {
  topProducts(first: Int): Products
  me: User
}

type User {
    id: ID
    name: String
    profilePic(size: Int): String
    status(membership: String): String
}

type Products {
  weight: Float
  size: Int
  name: String
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("variable `$undefinedVariable` is not defined"),
        "{errors}"
    );
    assert!(
        errors.contains("variable `$dimensions` is not defined"),
        "{errors}"
    );
}

#[test]
fn it_raises_unused_variable_in_query_error() {
    let input = r#"
query ExampleQuery($unusedVariable: Int) {
  topProducts {
    name
  }
  ... multipleSubscriptions
}

type Query {
  topProducts(first: Int): Product,
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("unused variable: `$unusedVariable`"),
        "{errors}"
    );
}

#[test]
fn it_raises_undefined_variable_in_query_in_fragments_error() {
    let input = r#"
query ExampleQuery {
  topProducts {
    name
  }

  me {
    ... on User {
      id
      name
      status(membership: $goldStatus)
    }
  }

  ... fragmentOne
}

fragment fragmentOne on Query {
    profilePic(size: $dimensions)
}

type Query {
  topProducts: Product
  profilePic(size: Int): String
  me: User
}

type User {
    id: ID
    name: String
    status(membership: String): String
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

    let errors = parse_mixed_validate(input, "schema.graphql")
        .unwrap_err()
        .to_string();
    assert!(
        errors.contains("variable `$goldStatus` is not defined"),
        "{errors}"
    );
    assert!(
        errors.contains("variable `$dimensions` is not defined"),
        "{errors}"
    );
}

/// apollo-parser already emits parse errors for variable syntax in const context,
/// but it is still possible to mutate Rust data structures to create `Value::Variable(x)`
/// that should be validation errors.
///
/// Here we parse a document that uses a string value `"x"` in all places a const value can show up
/// then programatically replace them with `$x` variable usage.
/// We expect the original document to be valid, and the modified documents to have
/// as many validation errors as occurrences of `"x"` strings in the original.
#[test]
fn variables_in_const_contexts() {
    let input = r#"
        directive @dir(
            arg: InputObj = {x: ["x"]} @dir2(arg: "x")
        ) repeatable on
            | QUERY
            | MUTATION
            | SUBSCRIPTION
            | FIELD
            | FRAGMENT_DEFINITION
            | FRAGMENT_SPREAD
            | INLINE_FRAGMENT
            | VARIABLE_DEFINITION
            | SCHEMA
            | SCALAR
            | OBJECT
            | FIELD_DEFINITION
            | ARGUMENT_DEFINITION
            | INTERFACE
            | UNION
            | ENUM
            | ENUM_VALUE
            | INPUT_OBJECT
            | INPUT_FIELD_DEFINITION

        directive @dir2(
            arg: String
        ) repeatable on
            | ARGUMENT_DEFINITION
            | INPUT_OBJECT
            | INPUT_FIELD_DEFINITION

        schema @dir(arg: {x: ["x"]}) {
            query: Query
        }
        extend schema @dir(arg: {x: ["x"]})

        scalar S @dir(arg: {x: ["x"]})
        extend scalar S @dir(arg: {x: ["x"]})

        type Query implements Inter @dir(arg: {x: ["x"]}) {
            field(
                arg1: String
                arg2: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            ): String @dir(arg: {x: ["x"]})
        }
        extend type Query @dir(arg: {x: ["x"]})

        interface Inter @dir(arg: {x: ["x"]}) {
            field(
                arg1: String
                arg2: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            ): String @dir(arg: {x: ["x"]})
        }
        extend interface Inter @dir(arg: {x: ["x"]})

        union U @dir(arg: {x: ["x"]}) = Query
        extend union U @dir(arg: {x: ["x"]})

        enum Maybe @dir(arg: {x: ["x"]}) {
            YES @dir(arg: {x: ["x"]})
            NO @dir(arg: {x: ["x"]})
        }
        extend enum Maybe @dir(arg: {x: ["x"]})

        input InputObj @dir2(arg: "x") {
            x: [String] = ["x"] @dir2(arg: "x")
        }
        extend input InputObj @dir2(arg: "x")

        query(
            $x: String
            $y: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
        ) {
            field(arg1: $x, arg2: $y)
        }
    "#;
    fn mutate_dir_arg(directive: &mut Node<ast::Directive>) {
        mutate_input_obj_value(&mut directive.make_mut().arguments[0].make_mut().value)
    }

    fn mutate_input_obj_value(value: &mut Node<Value>) {
        let Value::Object(fields) = value.make_mut() else {
            panic!("expected object")
        };
        let Value::List(items) = fields[0].1.make_mut() else {
            panic!("expected list")
        };
        mutate_string_value(&mut items[0])
    }

    fn mutate_string_value(value: &mut Node<Value>) {
        *value.make_mut() = Value::Variable(name!(x))
    }

    let (schema, doc) = apollo_compiler::parse_mixed_validate(input, "input.graphql").unwrap();
    let mut doc = doc.into_inner();

    let operation = doc.operations.anonymous.as_mut().unwrap().make_mut();
    let variable_def = operation.variables[1].make_mut();
    mutate_input_obj_value(variable_def.default_value.as_mut().unwrap());
    mutate_dir_arg(&mut variable_def.directives[0]);

    assert!(
        !doc.to_string().contains("\"x\""),
        "Did not replace all string values with variables:\n{}",
        doc
    );
    let errors = doc.validate(&schema).unwrap_err().errors;
    let expected = expect_test::expect![[r#"
        Error: variable `$x` is not defined
            ╭─[input.graphql:72:33]
            │
         72 │             $y: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            │                                 ─┬─  
            │                                  ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:72:54]
            │
         72 │             $y: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            │                                                      ─┬─  
            │                                                       ╰─── not found in this scope
        ────╯
    "#]];
    expected.assert_eq(&errors.to_string());
    let expected_executable_errors = 2;
    assert_eq!(errors.len(), expected_executable_errors);

    let mut schema = schema.into_inner();

    let dir_arg_def = schema.directive_definitions["dir"].make_mut().arguments[0].make_mut();
    let dir2 = dir_arg_def.directives[0].make_mut();
    mutate_input_obj_value(dir_arg_def.default_value.as_mut().unwrap());
    mutate_string_value(&mut dir2.arguments[0].make_mut().value);

    let def = schema.schema_definition.make_mut();
    mutate_dir_arg(&mut def.directives[0]);
    mutate_dir_arg(&mut def.directives[1]);

    let ExtendedType::Scalar(def) = &mut schema.types["S"] else {
        panic!("expected scalar")
    };
    let def = def.make_mut();
    mutate_dir_arg(&mut def.directives[0]);
    mutate_dir_arg(&mut def.directives[1]);

    let ExtendedType::Object(def) = &mut schema.types["Query"] else {
        panic!("expected object")
    };
    let def = def.make_mut();
    let field = def.fields[0].make_mut();
    let field_arg = field.arguments[1].make_mut();
    mutate_dir_arg(&mut def.directives[0]);
    mutate_dir_arg(&mut def.directives[1]);
    mutate_dir_arg(&mut field.directives[0]);
    mutate_dir_arg(&mut field_arg.directives[0]);
    mutate_input_obj_value(field_arg.default_value.as_mut().unwrap());

    let ExtendedType::Interface(def) = &mut schema.types["Inter"] else {
        panic!("expected interface")
    };
    let def = def.make_mut();
    let field = def.fields[0].make_mut();
    let field_arg = field.arguments[1].make_mut();
    mutate_dir_arg(&mut def.directives[0]);
    mutate_dir_arg(&mut def.directives[1]);
    mutate_dir_arg(&mut field.directives[0]);
    mutate_dir_arg(&mut field_arg.directives[0]);
    mutate_input_obj_value(field_arg.default_value.as_mut().unwrap());

    let ExtendedType::Union(def) = &mut schema.types["U"] else {
        panic!("expected union")
    };
    let def = def.make_mut();
    mutate_dir_arg(&mut def.directives[0]);
    mutate_dir_arg(&mut def.directives[1]);

    let ExtendedType::Enum(def) = &mut schema.types["Maybe"] else {
        panic!("expected enum")
    };
    let def = def.make_mut();
    mutate_dir_arg(&mut def.directives[0]);
    mutate_dir_arg(&mut def.directives[1]);
    mutate_dir_arg(&mut def.values["YES"].make_mut().directives[0]);
    mutate_dir_arg(&mut def.values["NO"].make_mut().directives[0]);

    let ExtendedType::InputObject(def) = &mut schema.types["InputObj"] else {
        panic!("expected input object")
    };
    let def = def.make_mut();
    let field = def.fields[0].make_mut();
    let Value::List(items) = field.default_value.as_mut().unwrap().make_mut() else {
        panic!("expected list")
    };
    mutate_string_value(&mut def.directives[0].make_mut().arguments[0].make_mut().value);
    mutate_string_value(&mut def.directives[1].make_mut().arguments[0].make_mut().value);
    mutate_string_value(&mut field.directives[0].make_mut().arguments[0].make_mut().value);
    mutate_string_value(&mut items[0]);

    assert!(
        !schema.to_string().contains("\"x\""),
        "Did not replace all string values with variables:\n{}",
        schema
    );
    let errors = schema.validate().unwrap_err().errors;
    let expected = expect_test::expect![[r#"
        Error: variable `$x` is not defined
           ╭─[input.graphql:3:34]
           │
         3 │             arg: InputObj = {x: ["x"]} @dir2(arg: "x")
           │                                  ─┬─  
           │                                   ╰─── not found in this scope
        ───╯
        Error: variable `$x` is not defined
           ╭─[input.graphql:3:51]
           │
         3 │             arg: InputObj = {x: ["x"]} @dir2(arg: "x")
           │                                                   ─┬─  
           │                                                    ╰─── not found in this scope
        ───╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:32:31]
            │
         32 │         schema @dir(arg: {x: ["x"]}) {
            │                               ─┬─  
            │                                ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:35:38]
            │
         35 │         extend schema @dir(arg: {x: ["x"]})
            │                                      ─┬─  
            │                                       ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:37:33]
            │
         37 │         scalar S @dir(arg: {x: ["x"]})
            │                                 ─┬─  
            │                                  ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:38:40]
            │
         38 │         extend scalar S @dir(arg: {x: ["x"]})
            │                                        ─┬─  
            │                                         ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:40:52]
            │
         40 │         type Query implements Inter @dir(arg: {x: ["x"]}) {
            │                                                    ─┬─  
            │                                                     ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:43:39]
            │
         43 │                 arg2: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            │                                       ─┬─  
            │                                        ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:43:60]
            │
         43 │                 arg2: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            │                                                            ─┬─  
            │                                                             ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:44:38]
            │
         44 │             ): String @dir(arg: {x: ["x"]})
            │                                      ─┬─  
            │                                       ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:46:42]
            │
         46 │         extend type Query @dir(arg: {x: ["x"]})
            │                                          ─┬─  
            │                                           ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:48:40]
            │
         48 │         interface Inter @dir(arg: {x: ["x"]}) {
            │                                        ─┬─  
            │                                         ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:51:39]
            │
         51 │                 arg2: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            │                                       ─┬─  
            │                                        ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:51:60]
            │
         51 │                 arg2: InputObj = {x: ["x"]} @dir(arg: {x: ["x"]})
            │                                                            ─┬─  
            │                                                             ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:52:38]
            │
         52 │             ): String @dir(arg: {x: ["x"]})
            │                                      ─┬─  
            │                                       ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:54:47]
            │
         54 │         extend interface Inter @dir(arg: {x: ["x"]})
            │                                               ─┬─  
            │                                                ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:56:32]
            │
         56 │         union U @dir(arg: {x: ["x"]}) = Query
            │                                ─┬─  
            │                                 ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:57:39]
            │
         57 │         extend union U @dir(arg: {x: ["x"]})
            │                                       ─┬─  
            │                                        ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:59:35]
            │
         59 │         enum Maybe @dir(arg: {x: ["x"]}) {
            │                                   ─┬─  
            │                                    ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:60:32]
            │
         60 │             YES @dir(arg: {x: ["x"]})
            │                                ─┬─  
            │                                 ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:61:31]
            │
         61 │             NO @dir(arg: {x: ["x"]})
            │                               ─┬─  
            │                                ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:63:42]
            │
         63 │         extend enum Maybe @dir(arg: {x: ["x"]})
            │                                          ─┬─  
            │                                           ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:65:35]
            │
         65 │         input InputObj @dir2(arg: "x") {
            │                                   ─┬─  
            │                                    ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:66:28]
            │
         66 │             x: [String] = ["x"] @dir2(arg: "x")
            │                            ─┬─  
            │                             ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:66:44]
            │
         66 │             x: [String] = ["x"] @dir2(arg: "x")
            │                                            ─┬─  
            │                                             ╰─── not found in this scope
        ────╯
        Error: variable `$x` is not defined
            ╭─[input.graphql:68:42]
            │
         68 │         extend input InputObj @dir2(arg: "x")
            │                                          ─┬─  
            │                                           ╰─── not found in this scope
        ────╯
    "#]];
    expected.assert_eq(&errors.to_string());
    let expected_schema_errors = 26;
    assert_eq!(errors.len(), expected_schema_errors);
    assert_eq!(
        input.matches("\"x\"").count(),
        expected_schema_errors + expected_executable_errors
    )
}

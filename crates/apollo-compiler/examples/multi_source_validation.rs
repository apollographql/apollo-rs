use std::{fs, io, path::Path};

use apollo_compiler::ApolloCompiler;

fn main() -> io::Result<()> {
    compile_from_dir()?;
    compile_schema_and_query_files()?;
    compile_schema_and_query_from_str()?;
    Ok(())
}

// Read all files regardless of what they are and validate them.
fn compile_from_dir() -> io::Result<()> {
    let mut compiler = ApolloCompiler::new();

    let dir = Path::new("crates/apollo-compiler/examples/documents");
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let src = fs::read_to_string(entry.path()).expect("Could not read document file.");
            compiler.add_document(&src, entry.path());
        }
    }

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert_eq!(diagnostics.len(), 1);

    Ok(())
}
// Read and set schemas and queries explicitly.
fn compile_schema_and_query_files() -> io::Result<()> {
    let mut compiler = ApolloCompiler::new();

    // add a schema file
    let schema = Path::new("crates/apollo-compiler/examples/documents/schema.graphql");
    let src = fs::read_to_string(schema).expect("Could not read schema file.");
    compiler.add_type_system(&src, schema);

    // schema_extension is still a file containing a type system, and it also gets added under .add_type_system API
    let schema_ext =
        Path::new("crates/apollo-compiler/examples/documents/schema_extension.graphql");
    let src = fs::read_to_string(schema_ext).expect("Could not read schema ext file.");
    compiler.add_type_system(&src, schema_ext);

    // get_dog_name is a query-only file and gets added with a .add_executable API
    let query = Path::new("crates/apollo-compiler/examples/documents/get_dog_name.graphql");
    let src = fs::read_to_string(query).expect("Could not read query file.");
    compiler.add_executable(&src, query);

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert_eq!(diagnostics.len(), 1);

    Ok(())
}

// Create apollo-compiler from schema and query str.
fn compile_schema_and_query_from_str() -> io::Result<()> {
    let schema = r#"
type Query {
  dog: Dog
}

enum DogCommand {
  SIT
  DOWN
  HEEL
}

type Dog implements Pet {
  name: String!
  nickname: String
  barkVolume: Int
  doesKnowCommand(dogCommand: DogCommand!): Boolean!
  isHouseTrained(atOtherHomes: Boolean): Boolean!
  owner: Human
}

interface Sentient {
  name: String!
}

interface Pet {
  name: String!
}

type Alien implements Sentient {
  name: String!
  homePlanet: String
}

type Human implements Sentient {
  name: String!
  pets: [Pet!]
}

enum CatCommand {
  JUMP
}

type Cat implements Pet {
  name: String!
  nickname: String
  doesKnowCommand(catCommand: CatCommand!): Boolean!
  meowVolume: Int
}

union CatOrDog = Cat | Dog
union DogOrHuman = Dog | Human
union HumanOrAlien = Human | Alien
    "#;

    let query = r#"
query getDogName {
  dog {
    name
  }
}

# duplicate name, should show up in diagnostics
query getDogName {
  dog {
    owner {
      name
    }
  }
}
    "#;
    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema, "schema.graphl");
    compiler.add_executable(query, "query.graphql");

    let diagnostics = compiler.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    assert_eq!(diagnostics.len(), 1);

    Ok(())
}

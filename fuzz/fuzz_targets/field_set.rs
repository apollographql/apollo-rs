#![no_main]
use apollo_compiler::executable::FieldSet;
use apollo_compiler::name;
use apollo_compiler::Schema;
use apollo_parser::cst::CstNode;
use apollo_parser::Parser;
use apollo_rs_fuzz::read_cst;
use libfuzzer_sys::fuzz_target;
use std::sync::LazyLock;

static SCHEMA: LazyLock<apollo_compiler::validation::Valid<Schema>> = LazyLock::new(|| {
    Schema::parse_and_validate("type Query { a: Int b(x: Int): Query }", "schema.graphql").unwrap()
});

// Parse arbitrary input as a field set. It has its own entry point with no
// document node around it, so document fuzzing doesn't cover it.
fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    read_cst(Parser::new(data).parse_selection_set().field_set().syntax());
    let limited = Parser::new(data)
        .recursion_limit(2)
        .token_limit(8)
        .parse_selection_set();
    read_cst(limited.field_set().syntax());

    let _ = FieldSet::parse(&SCHEMA, name!("Query"), data, "field_set.graphql");
});

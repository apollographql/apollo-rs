#![no_main]
use apollo_compiler::ast::Type;
use apollo_parser::cst::CstNode;
use apollo_parser::Parser;
use apollo_rs_fuzz::read_cst;
use libfuzzer_sys::fuzz_target;

// Parse arbitrary input as a type. It has its own entry point with no document
// node around it, so document fuzzing doesn't cover it.
fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    read_cst(Parser::new(data).parse_type().ty().syntax());
    let limited = Parser::new(data)
        .recursion_limit(2)
        .token_limit(4)
        .parse_type();
    read_cst(limited.ty().syntax());

    let _ = Type::parse(data, "type.graphql");
});

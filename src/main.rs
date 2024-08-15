use compiler::Compiler;
use object::ObjectOps;
use parser::{
    debug::pair_structure,
    grammar::{Rule, YsetlParser},
    parser::parse_expr_unwrap,
};
use vm::VM;

pub mod compiler;
pub mod object;
pub mod op;
pub mod parser;
pub mod vm;

pub fn print_structure(rule: Rule, input: &str) {
    use pest::Parser;
    let result = YsetlParser::parse(rule, input).unwrap().next().unwrap();
    println!("{}", pair_structure(result));
}

fn main() {
    let ast = parse_expr_unwrap("3 + 4 + 8");
    let mut comp = Compiler::new();
    comp.compile_expr(ast);
    let bc = comp.finish();
    println!("Results:");
    println!("Instructions: {:?}", bc.instructions);
    println!("Constants: {:?}", bc.constants);

    let vm = VM::new(bc);
    let result = vm.run();
    println!("Evaluates to: {}", result.unwrap().to_s());
}

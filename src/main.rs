use compiler::Compiler;
use parser::{
    debug::pair_structure,
    grammar::{Rule, YsetlParser},
    parser::parse_program,
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
    let ast = parse_program("print 11 + 13 ** -1;\nprint -3 + 4 + 8;");
    let comp = Compiler::new();
    let bc = comp.compile_program(ast);
    println!("Results:");
    println!("Constants: {:?}", bc.constants);
    println!("\n:: START INSTRUCTIONS ::");
    bc.print_bytecode();
    println!("::  END INSTRUCTIONS  ::\n");

    println!(":: START EXECUTION ::");
    let vm = VM::new(bc);
    vm.run();
    println!(":: DONE EXECUTION  ::");
}

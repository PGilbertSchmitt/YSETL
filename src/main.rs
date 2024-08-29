use compiler::{bytecode::print_bytecode, compiler::Compiler};
use object::object::BaseObject;
use parser::{
    debug::pair_structure,
    grammar::{Rule, YsetlParser},
    parser::parse_program,
};
use pest::Parser;
use vm::vm::VM;

pub mod compiler;
pub mod object;
pub mod op;
pub mod parser;
pub mod vm;

pub fn print_structure(rule: Rule, input: &str) {
    let result = YsetlParser::parse(rule, input).unwrap().next().unwrap();
    println!("{}", pair_structure(result));
}

fn main() {
    let ast = parse_program("
        const = 55;
        foo = (a) => (b) => (c) => a + b + c + const;
        print foo(3)(6)(5);
    ");
    let comp = Compiler::new();
    let bc = comp.compile_program(ast);
    println!("Results:");
    println!("Constants: {:?}", bc.constants);
    println!("\n:: START INSTRUCTIONS ::");
    bc.print_bytecode();
    println!("::  END INSTRUCTIONS  ::\n");

    for (i, c) in bc.constants.iter().enumerate() {
        match c {
            BaseObject::Closure(cl) => {
                println!(":: START CLOSURE INSTRUCTIONS [{i}] ::");
                print_bytecode(&cl.ins);
                println!("::  END CLOSURE INSTRUCTIONS  [{i}] ::\n");
            },
            _ => ()
        }
    }

    println!(":: START EXECUTION ::");
    let vm = VM::new(bc);
    vm.run();
    println!(":: DONE EXECUTION  ::");
}

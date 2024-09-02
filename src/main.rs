use compiler::{bytecode::print_bytecode, compiler::Compiler};
use object::object::Object;
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
    let ast = parse_program(
        "
        A = [2..100];
        foo = (val) => choose x, y, z in A -> x * y * z == val;
        print foo(696);
        print foo(1111);
        print {1,{2,{4,{4,{4,{4,{4,{4,{4,{4,{2},7},7},7},7},7},7},7},7},7}};
    ",
    );
    let comp = Compiler::new();
    let bc = comp.compile_program(ast);
    println!("Results:");
    println!("Constants: {:?}", bc.constants);
    println!("\n:: START INSTRUCTIONS ::");
    bc.print_bytecode();
    println!("::  END INSTRUCTIONS  ::\n");

    for (i, c) in bc.constants.iter().enumerate() {
        match c {
            Object::Closure { inner, .. } => {
                println!(":: START CLOSURE INSTRUCTIONS [{i}] ::");
                print_bytecode(&inner.executor.ins);
                println!("::  END CLOSURE INSTRUCTIONS  [{i}] ::\n");
            }
            _ => (),
        }
    }

    for (idx, iter) in bc.iterators.iter().enumerate() {
        println!(":: START ITERATOR INSTRUCTIONS [{idx}] ::");
        print_bytecode(&iter.ins);
        println!("::  END ITERATOR INSTRUCTIONS  [{idx}] ::\n");
    }

    println!(":: START EXECUTION ::");
    let vm = VM::new(bc);
    vm.run();
    println!(":: DONE EXECUTION  ::");
}

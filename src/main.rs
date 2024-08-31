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
    let ast = parse_program(
        "   
        B = [1..10];
        foo = (A) => [x+y : x in B, y in A -> x < 5 && y % 2 == 0];
        print foo([1..5])
    ",
    );
    // let ast = parse_program("
    //     foo = (a) => {print a;  foo(a+1)};
    //     print foo(1);
    // ");
    let comp = Compiler::new();
    let bc = comp.compile_program(ast);
    println!("Results:");
    println!("Constants: {:?}", bc.constants);
    println!("\n:: START INSTRUCTIONS ::");
    bc.print_bytecode();
    println!("::  END INSTRUCTIONS  ::\n");

    for (i, c) in bc.constants.iter().enumerate() {
        match c {
            BaseObject::Closure { function, .. } => {
                println!(":: START CLOSURE INSTRUCTIONS [{i}] ::");
                print_bytecode(&function.ins);
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

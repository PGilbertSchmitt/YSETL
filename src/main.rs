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
        "\
A = [1..500000];
print #[{(x): x*2} :: x in A];
print #{{(x): x*2} :: x in A};
print #{x, x*2 :: x in A};
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
    let start_time = std::time::Instant::now();
    vm.run();
    println!(
        ":: DONE EXECUTION  :: (took {}ms)",
        start_time.elapsed().as_millis()
    );
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        hash::{Hash, Hasher},
    };

    use nohash_hasher::{self, BuildNoHashHasher};
    use xxhash_rust::xxh3::Xxh3;

    #[derive(Eq, PartialEq)]
    enum Foo {
        N,
        A(u8),
        B(String),
        C(Vec<Foo>),
        D(bool),
    }

    impl nohash_hasher::IsEnabled for Foo {}

    impl Hash for Foo {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            match self {
                Self::N => {
                    state.write_u8(b'n');
                }
                Self::A(x) => {
                    state.write_u8(*x);
                }
                Self::B(s) => {
                    let mut xh = Xxh3::new();
                    xh.update(s.as_bytes());
                    xh.write_u8(b's');
                    state.write_u64(xh.digest());
                }
                Self::C(v) => {
                    let mut xh = Xxh3::new();
                    v.hash(&mut xh);
                    xh.write_u8(b'v');
                    state.write_u64(xh.digest());
                }
                Self::D(b) => b.hash(state),
            }
        }
    }

    #[test]
    fn no_hash_hashing() {
        let mut m = HashMap::<Foo, u64, BuildNoHashHasher<u64>>::with_capacity_and_hasher(
            8,
            BuildNoHashHasher::default(),
        );
        m.insert(Foo::N, 5);
        m.insert(Foo::A(25), 10);
        m.insert(Foo::B(String::from("n")), 15);
        m.insert(Foo::C(vec![Foo::N]), 20);
        m.insert(Foo::D(true), 25);
        m.insert(Foo::D(false), 30);
    }
}

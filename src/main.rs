use parser::{
    debug::pair_structure,
    grammar::{Rule, YsetlParser}, parser::parse,
};

pub mod parser;

pub fn print_structure(rule: Rule, input: &str) {
    use pest::Parser;
    let result = YsetlParser::parse(rule, input)
        .unwrap()
        .next()
        .unwrap();
    println!("{}", pair_structure(result));
}

fn main() {
    parse("switch (something) {
        case x then :x,
        case exists y in z where y == 2 then :y,
        case ~ |> :z,
    }");
    parse("switch {
        case x then :x,
        case exists y in z where y == 2 then :y,
        case ~ |> :z,
    }");
}

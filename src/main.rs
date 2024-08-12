use parser::{
    debug::pair_structure,
    grammar::{Rule, YsetlParser},
    parser::parse,
};

pub mod parser;

pub fn print_structure(rule: Rule, input: &str) {
    use pest::Parser;
    let result = YsetlParser::parse(rule, input).unwrap().next().unwrap();
    println!("{}", pair_structure(result));
}

fn main() {
    parse("foo[a..b]");
    parse("foo[a..]");
    parse("foo[..b]");
    parse("foo[...]");
}

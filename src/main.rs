use parser::{grammar::Rule, parser::{parse, print_structure}};

pub mod parser;

fn main() {
    print_structure(Rule::expr_list, ":a, :a");
}

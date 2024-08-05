use parser::{grammar::Rule, parser::print_structure};

pub mod parser;

fn main() {
    print_structure(Rule::expr_list, ":a, :a");
}

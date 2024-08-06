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
    println!("{}", pair_structure(result))
}

fn main() {
    print_structure(Rule::expr, "{x:[x]=foo(y)}");
    parse("{x+y:x,y in[a+b..b+a**2], a=foo( b), b = bar{g} | x > k, g<nu}");
}

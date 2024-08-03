use parser::parser::parse;

pub mod parser;

fn main() {
    parse(":abc");
    parse(":123");
}

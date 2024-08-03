use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/ysetl.pest"]
pub struct YsetlParser;

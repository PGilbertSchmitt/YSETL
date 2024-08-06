use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/ysetl.pest"]
pub struct YsetlParser;

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::{Rule, YsetlParser};

    fn parse_is_ok(rule: Rule, input: &str) {
        match YsetlParser::parse(rule, input) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false, "Error parsing rule '{rule}' for input '{input}'")
        }
    }

    #[test]
    fn keyword_literals() {
        parse_is_ok(Rule::kw_null, "null");
        parse_is_ok(Rule::kw_newat, "newat");
        parse_is_ok(Rule::kw_false, "false");
        parse_is_ok(Rule::kw_true, "true");
    }

    #[test]
    fn atom_literal() {
        parse_is_ok(Rule::atom, ":abcd");
    }

    #[test]
    fn ident_literal() {
        parse_is_ok(Rule::ident, "abcd");
        parse_is_ok(Rule::ident, "abc2");
        parse_is_ok(Rule::ident, "ab2_cd3");
    }

    #[test]
    fn number_literal() {
        parse_is_ok(Rule::number, "1");
        parse_is_ok(Rule::number, "123.456");
        parse_is_ok(Rule::number, "1.23456e-2");
        parse_is_ok(Rule::number, "01e2");
        parse_is_ok(Rule::number, "01f2");
        parse_is_ok(Rule::number, "01E2");
        parse_is_ok(Rule::number, "01F2");
    }

    #[test]
    fn string_literal() {
        parse_is_ok(Rule::string, "\"hello, world\"");
        parse_is_ok(Rule::string, "\"Hello. \\nWorld.\"");
        parse_is_ok(Rule::string, "\"  hello, \\\"world\\\"\"  ");
    }

    #[test]
    fn tuple_literal() {
        parse_is_ok(Rule::tuple_literal, "[]");
        parse_is_ok(Rule::tuple_literal, "[1]");
        parse_is_ok(Rule::tuple_literal, "[ 1 ]");
        parse_is_ok(Rule::tuple_literal, "[1,2]");
        parse_is_ok(Rule::tuple_literal, "[1..10]");
        parse_is_ok(Rule::tuple_literal, "[1,3..10]");
        // parse_is_ok(Rule::tuple_literal, "[x+2 : x in Z]");
        // parse_is_ok(Rule::tuple_literal, "[[x,y] : x in Z, y=W(x) | not x]");
    }

    #[test]
    fn set_literal() {
        parse_is_ok(Rule::set_literal, "{}");
        parse_is_ok(Rule::set_literal, "{1}");
        parse_is_ok(Rule::set_literal, "{1,2}");
        parse_is_ok(Rule::set_literal, "{1..10}");
        parse_is_ok(Rule::set_literal, "{1,3..10}");
        // parse_is_ok(Rule::tuple_literal, "{x+2 : x in Z}");
        // parse_is_ok(Rule::tuple_literal, "{[x,y] : x in Z, y=W(x) | not x}");
    }

    #[test]
    fn iterators() {
        parse_is_ok(Rule::iterator, "x in Z");
        parse_is_ok(Rule::iterator, "x=f(y)");
        parse_is_ok(Rule::iterator, "[x,y]=f(z)");
        parse_is_ok(Rule::iterator, "x=Z(y),a=C(b)");
        parse_is_ok(Rule::iterator, "x in Z | not x");
        parse_is_ok(Rule::iterator, "x,y in[a+b..b+a**2], a=foo(b), b=bar{g} | x > k, g < u");
    }
}

use lazy_static;
use pest::iterators::Pair;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use pest::Span;

use super::ast::BinOp;
use super::ast::ExprST;
use super::ast::PreOp;
use super::grammar::Rule;
use super::grammar::YsetlParser;

type ExprResult = Result<ExprST, String>;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Op, Assoc::{Right, Left}};

        PrattParser::new()
            // Keyword operators have the lowest precedence, but their relative
            // precedence is similar to the symbol equivalents
            .op(Op::infix(Rule::kw_iff, Left))
            .op(Op::infix(Rule::kw_impl, Left))
            .op(Op::infix(Rule::kw_or, Left))
            .op(Op::infix(Rule::kw_and, Left))
            .op(Op::infix(Rule::kw_in, Left) |
                Op::infix(Rule::kw_notin, Left) |
                Op::infix(Rule::kw_subset, Left))
            .op(Op::infix(Rule::kw_with, Left) |
                Op::infix(Rule::kw_less, Left) |
                Op::infix(Rule::kw_union, Left))
            .op(Op::infix(Rule::kw_mod, Left) |
                Op::infix(Rule::kw_div, Left) |
                Op::infix(Rule::kw_inter, Left))
            .op(Op::prefix(Rule::kw_not))
            // Then the symbol operators all behave as normal
            // Frankly, I don't want to stress too much about this,
            // so I'm just copying precendence rules from C++ and Ruby
            // (what a mix)
            .op(Op::infix(Rule::dbl_pipe, Left))
            .op(Op::infix(Rule::dbl_amp, Left))
            .op(Op::infix(Rule::pipe, Left))
            .op(Op::infix(Rule::caret, Left))
            .op(Op::infix(Rule::amp, Left))
            .op(Op::infix(Rule::dbl_eq, Left) |
                Op::infix(Rule::bang_eq, Left))
            .op(Op::infix(Rule::lt, Left) |
                Op::infix(Rule::lt_eq, Left) |
                Op::infix(Rule::gt, Left) |
                Op::infix(Rule::gt_eq, Left))
            .op(Op::infix(Rule::dbl_lt, Left) |
                Op::infix(Rule::dbl_gt, Left))
            // Infix-injector operator
            .op(Op::infix(Rule::plus, Left) |
                Op::infix(Rule::dash, Left))
            .op(Op::infix(Rule::star, Left) |
                Op::infix(Rule::slash, Left))
            .op(Op::infix(Rule::dbl_star, Right))
            // Reduce operator
            .op(Op::infix(Rule::dbl_qst, Right))
            .op(Op::infix(Rule::at, Right))
            .op(Op::prefix(Rule::dash) |
                Op::prefix(Rule::hash) |
                Op::prefix(Rule::bang) |
                Op::prefix(Rule::caret) |
                Op::prefix(Rule::dollar) |
                Op::prefix(Rule::tilde) |
                Op::prefix(Rule::amp))
    };
}

pub fn print_structure(input: &str) {
    let result = YsetlParser::parse(Rule::expr, input)
        .unwrap()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    println!("{}", result)
}

pub fn parse(input: &str) {
    let result = YsetlParser::parse(Rule::expr, input)
        .unwrap()
        .next()
        .unwrap();

    match parse_expr(result) {
        Ok(expr) => println!("{:?}", expr),
        Err(reason) => println!("{reason}"),
    }

    // match expr.as_rule() {
    //     Rule::bin_expr => {
    //         eval_infix(expr);
    //     }
    //     _ => unimplemented!(),
    // }
}

fn parse_expr(expression: Pair<Rule>) -> ExprResult {
    PRATT_PARSER
        .map_primary(parse_primary)
        .map_prefix(parse_pre_op)
        .map_infix(parse_infix)
        .parse(expression.into_inner())
}

fn parse_primary(primary: Pair<Rule>) -> ExprResult {
    match primary.as_rule() {
        Rule::kw_null => Ok(ExprST::Null),
        Rule::kw_newat => Ok(ExprST::Newat),
        Rule::kw_true => Ok(ExprST::True),
        Rule::kw_false => Ok(ExprST::False),
        Rule::string => parse_string(primary),
        Rule::atom => parse_atom(primary),
        Rule::number => parse_number(primary),
        _ => unimplemented!(),
    }
}

// String literal primary parses into `string(s, e, [string_keep(s+1, e-1)])`
fn parse_string(pair: Pair<Rule>) -> ExprResult {
    let inner = pair.into_inner();
    Ok(ExprST::String(inner.as_str().to_owned()))
}

// Atom literal primary parses into `atom(s, e, [atom_keep(s+1, e)])`
fn parse_atom(pair: Pair<Rule>) -> ExprResult {
    let inner = pair.into_inner();
    Ok(ExprST::Atom(inner.as_str().to_owned()))
}

// Number literal primary parses into
// `number(s, e, [number_base(s, _), number_decimal(_, _), number_exp(_, e)])`
fn parse_number(pair: Pair<Rule>) -> ExprResult {
    let span = pair.as_span();
    let mut parts = pair.into_inner().map(|part| part.as_str());
    let base = parts.next().unwrap();
    let decimal = parts.next().unwrap();
    let exponent = parts.next().unwrap();
    let with_exponent = !exponent.is_empty();

    let is_float = !decimal.is_empty() || with_exponent;
    let mut number_string = base.to_owned();

    number_string.push_str(decimal);

    if with_exponent {
        number_string.push('e');
        number_string.push_str(&exponent[1..]);
    }

    if is_float {
        let parse_result: Result<f64, std::num::ParseFloatError> = number_string.parse::<f64>();
        match parse_result {
            Ok(float) => {
                if float.is_infinite() {
                    let span_string = span_start_str(span);
                    Err(format!("{span_string}, Encountered float parsing error: float literal too large to process"))
                } else {
                    Ok(ExprST::Float(float))
                }
            }
            Err(err) => {
                let span_string = span_start_str(span);
                Err(format!(
                    "{span_string}, Encountered float parsing error: {}",
                    err.to_string()
                ))
            }
        }
    } else {
        number_string
            .parse::<i64>()
            .map(|int| ExprST::Integer(int))
            .map_err(|err| {
                let span_string = span_start_str(span);
                format!(
                    "{span_string}, Encountered integer parsing error: {}",
                    err.to_string()
                )
            })
    }
}

fn parse_pre_op(op: Pair<Rule>, rhs: ExprResult) -> ExprResult {
    let op = match op.as_rule() {
        Rule::kw_not | Rule::bang => PreOp::Not,
        Rule::dash => PreOp::Negate,
        Rule::plus => PreOp::Identity,
        Rule::hash => PreOp::Size,
        Rule::caret => PreOp::Head,
        Rule::dollar => PreOp::Last,
        Rule::tilde => PreOp::Tail,
        Rule::amp => PreOp::Init,
        _ => unimplemented!(),
    };
    Ok(ExprST::Prefix {
        op: op,
        rhs: Box::new(rhs?),
    })
}

fn parse_infix(lhs: ExprResult, op: Pair<Rule>, rhs: ExprResult) -> ExprResult {
    let op = match op.as_rule() {
        Rule::dbl_qst => BinOp::Nullcoel,
        Rule::dbl_star => BinOp::Exp,
        Rule::dbl_lt | Rule::kw_with => BinOp::WithBitLeft,
        Rule::dbl_gt | Rule::kw_less => BinOp::LessBitRight,
        Rule::dbl_eq | Rule::kw_iff => BinOp::Eq,
        Rule::dbl_amp | Rule::kw_and => BinOp::Add,
        Rule::dbl_pipe | Rule::kw_or => BinOp::Or,
        Rule::lt => BinOp::Lt,
        Rule::gt => BinOp::Gt,
        Rule::lt_eq => BinOp::Lteq,
        Rule::gt_eq => BinOp::Gteq,
        Rule::bang_eq => BinOp::Neq,
        Rule::at => BinOp::Take,
        Rule::star | Rule::kw_inter => BinOp::Mult,
        Rule::slash | Rule::kw_div => BinOp::Div,
        Rule::plus | Rule::kw_union => BinOp::Add,
        Rule::dash => BinOp::Subtract,
        Rule::amp => BinOp::BitAnd,
        Rule::pipe => BinOp::BitOr,
        Rule::caret => BinOp::BitXor,
        Rule::kw_impl => BinOp::Impl,
        Rule::kw_mod => BinOp::Mod,
        Rule::kw_in => BinOp::In,
        Rule::kw_notin => BinOp::Notin,
        Rule::kw_subset => BinOp::Subset,
        _ => unreachable!(),
    };
    Ok(ExprST::Infix {
        op: op,
        lfs: Box::new(lhs?),
        rhs: Box::new(rhs?),
    })
}

fn span_start_str(span: Span) -> String {
    let (line, col) = span.start_pos().line_col();
    format!("Line {}, Col: {}", line, col)
}

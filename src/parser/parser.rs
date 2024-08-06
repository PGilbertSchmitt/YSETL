use lazy_static;
use pest::iterators::Pair;
use pest::pratt_parser::PrattParser;
use pest::Parser;
use pest::Span;

use super::ast::BinOp;
use super::ast::Bound;
use super::ast::BoundList;
use super::ast::Expr;
use super::ast::ExprList;
use super::ast::Former;
use super::ast::Iterator;
use super::ast::IteratorList;
use super::ast::PreOp;
use super::ast::SingleIterator;
use super::grammar::Rule;
use super::grammar::YsetlParser;

// TODO: Support better Err type with location information
type YsetlParseError = String;

type ExprResult = Result<Expr, YsetlParseError>;
type FormerResult = Result<Former, YsetlParseError>;
type VecResult = Result<ExprList, YsetlParseError>;
type SingleIteratorResult = Result<SingleIterator, YsetlParseError>;

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

pub fn parse(input: &str) {
    let result = YsetlParser::parse(Rule::expr, input)
        .unwrap()
        .next()
        .unwrap();

    match parse_expr(result) {
        Ok(expr) => println!("output -> {:?}", expr),
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
        Rule::kw_null => Ok(Expr::Null),
        Rule::kw_newat => Ok(Expr::Newat),
        Rule::kw_true => Ok(Expr::True),
        Rule::kw_false => Ok(Expr::False),
        Rule::string => parse_string(primary),
        Rule::atom => parse_atom(primary),
        Rule::ident => parse_ident(primary),
        Rule::number => parse_number(primary),
        Rule::tuple_literal => parse_tuple_literal(primary),
        Rule::set_literal => parse_set_literal(primary),
        rule => {
            println!("failed to process rule: {:?}", rule);
            unimplemented!()
        }
    }
}

// string([string_keep])
fn parse_string(pair: Pair<Rule>) -> ExprResult {
    let inner = pair.into_inner();
    Ok(Expr::String(inner.as_str().to_owned()))
}

// atom([atom_keep])
fn parse_atom(pair: Pair<Rule>) -> ExprResult {
    let inner = pair.into_inner();
    Ok(Expr::Atom(inner.as_str().to_owned()))
}

fn parse_ident(pair: Pair<Rule>) -> ExprResult {
    Ok(Expr::Ident(pair.as_str().to_owned()))
}

// number([number_base, number_decimal, number_exp])
fn parse_number(pair: Pair<Rule>) -> ExprResult {
    let span = pair.as_span();
    let mut parts = pair.into_inner();//.map(|part| part.as_str());
    let base = careful_unwrap(parts.next())?.as_str();
    let decimal = careful_unwrap(parts.next())?.as_str();
    let exponent = careful_unwrap(parts.next())?.as_str();
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
                    Ok(Expr::Float(float))
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
            .map(|int| Expr::Integer(int))
            .map_err(|err| {
                let span_string = span_start_str(span);
                format!(
                    "{span_string}, Encountered integer parsing error: {}",
                    err.to_string()
                )
            })
    }
}

// tuple_literal([FORMER])
fn parse_tuple_literal(pair: Pair<Rule>) -> ExprResult {
    Ok(Expr::Tuple(parse_former(pair)?))
}

// set_literal([FORMER])
fn parse_set_literal(pair: Pair<Rule>) -> ExprResult {
    Ok(Expr::Set(parse_former(pair)?))
}

fn parse_former(pair: Pair<Rule>) -> FormerResult {
    if let Some(former) = pair.into_inner().next() {
        match former.as_rule() {
            Rule::expr_list => Ok(Former::Literal(parse_expr_list(former)?)),
            Rule::range_former => parse_range_former(former),
            Rule::interval_range_former => parse_interval_range_former(former),
            Rule::iterator_former => parse_iterator_former(former),
            _ => unreachable!(),
        }
    } else {
        Ok(Former::Empty)
    }
}

// expr_list([EXPR, EXPR, ..., EXPR])
fn parse_expr_list(pair: Pair<Rule>) -> VecResult {
    pair.into_inner().map(|inner| parse_expr(inner)).collect()
}

// range_former([EXPR, exclusive_range_op, EXPR])
// range_former([EXPR, inclusive_range_op, EXPR])
fn parse_range_former(pair: Pair<Rule>) -> FormerResult {
    let mut parts = pair.into_inner();
    let start = parse_expr(careful_unwrap(parts.next())?)?;
    let op = careful_unwrap(parts.next())?;
    let end = parse_expr(careful_unwrap(parts.next())?)?;

    let inclusive = match op.as_rule() {
        Rule::inclusive_range_op => true,
        Rule::exclusive_range_op => false,
        _ => unreachable!(),
    };

    Ok(Former::Range {
        inclusive: inclusive,
        start: Box::new(start),
        end: Box::new(end),
        step: None,
    })
}

// interval_range_former([EXPR, RANGE_FORMER])
fn parse_interval_range_former(pair: Pair<Rule>) -> FormerResult {
    let mut parts = pair.into_inner();
    let step = parse_expr(careful_unwrap(parts.next())?)?;
    let range = parse_range_former(careful_unwrap(parts.next())?)?;
    if let Former::Range {
        inclusive,
        start,
        end,
        ..
    } = range
    {
        Ok(Former::Range {
            inclusive: inclusive,
            start: start,
            end: end,
            step: Some(Box::new(step)),
        })
    } else {
        Err(String::from(
            "Unexpected error while parsing interval range",
        ))
    }
}

fn parse_iterator_former(pair: Pair<Rule>) -> FormerResult {
    let mut parts = pair.into_inner();
    let expr = parse_expr(careful_unwrap(parts.next())?)?;
    let iterator = parse_iterator(careful_unwrap(parts.next())?)?;
    Ok(Former::Iterator { output: Box::new(expr), iterator: iterator })
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
    Ok(Expr::Prefix {
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
    Ok(Expr::Infix {
        op: op,
        lfs: Box::new(lhs?),
        rhs: Box::new(rhs?),
    })
}

fn parse_bound(pair: Pair<Rule>) -> Bound {
    match pair.as_rule() {
        Rule::tilde => Bound::Tilde,
        Rule::ident => Bound::Ident(pair.as_str().to_owned()),
        Rule::bound_list => Bound::List(parse_bound_list(pair)),
        _ => unreachable!(),
    }
}

// bound_list([BOUND, BOUND, ..., BOUND])
fn parse_bound_list(pair: Pair<Rule>) -> BoundList {
    pair.into_inner().map(parse_bound).collect()
}

//  iterator([ITERATOR_LIST, EXPR_LIST?])
fn parse_iterator(pair: Pair<Rule>) -> Result<Iterator, YsetlParseError> {
    let mut parts = pair.into_inner();
    let iterator_list = parse_iterator_list(careful_unwrap(parts.next())?)?;
    let filters = match parts.next() {
        Some(exprs) => parse_expr_list(exprs)?,
        None => vec![],
    };
    Ok(Iterator {
        iterators: iterator_list,
        filters: filters,
    })
}

// iterator_list([SINGLE_ITERATOR, SINGLE_ITERATOR, ..., SINGLE_ITERATOR])
fn parse_iterator_list(pair: Pair<Rule>) -> Result<IteratorList, YsetlParseError> {
    pair.into_inner().map(parse_single_iterator).collect()
}

fn parse_single_iterator(pair: Pair<Rule>) -> SingleIteratorResult {
    match pair.as_rule() {
        Rule::in_iterator => parse_in_iterator(pair),
        Rule::select_iterator_single |
        Rule::select_iterator_multi => parse_select_iterator(pair),
        _ => unreachable!()
    }
}

// in_iterator([BOUND_LIST, EXPR])
fn parse_in_iterator(pair: Pair<Rule>) -> SingleIteratorResult {
    let mut parts = pair.into_inner();
    let bound_list = parse_bound_list(careful_unwrap(parts.next())?);
    let expr = parse_expr(careful_unwrap(parts.next())?)?;
    Ok(SingleIterator::In { bounds: bound_list, expr: expr })
}

// select_iterator_single([BOUND, IDENT, BOUND_LIST])
// select_iterator_multi([BOUND, IDENT, BOUND_LIST])
fn parse_select_iterator(pair: Pair<Rule>) -> SingleIteratorResult {
    let single = pair.as_rule() == Rule::select_iterator_single;
    let mut parts = pair.into_inner();
    let bound = parse_bound(careful_unwrap(parts.next())?);
    let collection_ident = careful_unwrap(parts.next())?.as_str().to_owned();
    let bound_list = parse_bound_list(careful_unwrap(parts.next())?);
    if single {
        Ok(SingleIterator::SelectOne {
            bound: bound,
            collection: collection_ident,
            list: bound_list
        })
    } else {
        Ok(SingleIterator::SelectMany {
            bound: bound,
            collection: collection_ident,
            list: bound_list
        })
    }
}

fn span_start_str(span: Span) -> String {
    let (line, col) = span.start_pos().line_col();
    format!("Line {}, Col: {}", line, col)
}

fn careful_unwrap(part: Option<Pair<Rule>>) -> Result<Pair<Rule>, YsetlParseError> {
    part.map_or_else(
        || Err(String::from("Something horrific and unexpected has occured, please consult your doctor.")),
        |part| Ok(part),
    )
}

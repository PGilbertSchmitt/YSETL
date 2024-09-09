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
use super::ast::Postfix;
use super::ast::PreOp;
use super::ast::Range;
use super::ast::SelectOp;
use super::ast::SingleIterator;
use super::ast::Stmt;
use super::ast::StmtList;
use super::ast::StmtListWithCapture;
use super::ast::SwitchCase;
use super::grammar::Rule;
use super::grammar::YsetlParser;

// TODO: Support better Err type with location information
type YsetlParseError = String;
type YsetlParseResult<T> = Result<T, YsetlParseError>;

type StmtResult = YsetlParseResult<Stmt>;
type ExprResult = YsetlParseResult<Expr>;
type FormerResult = YsetlParseResult<Former>;
type StmtListResult = YsetlParseResult<StmtList>;
type ExprListResult = YsetlParseResult<ExprList>;
type SingleIteratorResult = YsetlParseResult<SingleIterator>;
type CaseResult = YsetlParseResult<SwitchCase>;
type PostfixResult = YsetlParseResult<Postfix>;
type BlockResult = YsetlParseResult<StmtListWithCapture>;

type ParamList = (Vec<String>, Vec<String>);

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
            .op(Op::infix(Rule::infix_inject_op, Right))
            .op(Op::infix(Rule::plus, Left) |
                Op::infix(Rule::dash, Left))
            .op(Op::infix(Rule::star, Left) |
                Op::infix(Rule::slash, Left) |
                Op::infix(Rule::percent, Left))
            .op(Op::infix(Rule::dbl_star, Right))
            .op(Op::infix(Rule::reduce_op, Right))
            .op(Op::infix(Rule::dbl_qst, Right))
            .op(Op::infix(Rule::at, Right))
            .op(Op::prefix(Rule::dash_pre) |
                Op::prefix(Rule::hash) |
                Op::prefix(Rule::bang) |
                Op::prefix(Rule::caret_pre) |
                Op::prefix(Rule::dollar) |
                Op::prefix(Rule::tilde) |
                Op::prefix(Rule::amp_pre))
            .op(Op::postfix(Rule::fn_call) |
                Op::postfix(Rule::index_call) |
                Op::postfix(Rule::slice_call) |
                Op::postfix(Rule::pick_call))
    };
}

pub fn parse_program(input: &str) -> StmtList {
    let result = YsetlParser::parse(Rule::raw_program, input)
        .unwrap()
        .next()
        .unwrap()
        .into_inner()
        .next()
        .unwrap();

    let stmt_list_with_capture = parse_stmt_list_with_capture(result).unwrap();
    let mut list = stmt_list_with_capture.stmt_list;
    if let Some(expr_box) = stmt_list_with_capture.implicit_return {
        list.push(*expr_box);
    }
    list
}

fn parse_stmt(stmt: Pair<Rule>) -> StmtResult {
    match stmt.as_rule() {
        Rule::return_stmt => parse_return_stmt(stmt),
        Rule::print_stmt => parse_print_stmt(stmt),
        // Rule::cond_stmt => parse_cond_stmt(stmt),
        Rule::assign_stmt => parse_assign_stmt(stmt),
        _ => Ok(Stmt::Expr(parse_expr(stmt)?)),
    }
}

pub fn parse_expr(expr: Pair<Rule>) -> ExprResult {
    match expr.as_rule() {
        Rule::nested_expr => parse_nested_expr(expr),
        Rule::select_expr => parse_select_expr(expr),
        Rule::ternary_expr => parse_ternary_expr(expr),
        Rule::switch_expr => parse_switch_expr(expr),
        Rule::gen_expr => parse_bin_expr(expr),
        Rule::block_expr => parse_block_expr(expr),
        _ => panic!("Failed to process rule {} as expr", expr),
    }
}

fn parse_nested_expr(expr: Pair<Rule>) -> ExprResult {
    let inner = careful_unwrap(expr.into_inner().next())?;
    parse_expr(inner)
}

fn parse_bin_expr(bin_expr: Pair<Rule>) -> ExprResult {
    PRATT_PARSER
        .map_primary(parse_primary)
        .map_prefix(parse_pre_op)
        .map_postfix(parse_postfix)
        .map_infix(parse_infix)
        .parse(bin_expr.into_inner())
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
        Rule::func_literal => parse_function_literal(primary),
        Rule::nested_expr => parse_nested_expr(primary),
        _ => {
            panic!("failed to process rule {} as primary", primary);
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
    let mut parts = pair.into_inner();
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
fn parse_expr_list(pair: Pair<Rule>) -> ExprListResult {
    pair.into_inner().map(|inner| parse_expr(inner)).collect()
}

fn parse_range_former(pair: Pair<Rule>) -> FormerResult {
    Ok(Former::Range(parse_range(pair)?))
}

// range_former([EXPR, exclusive_range_op, EXPR])
// range_former([EXPR, inclusive_range_op, EXPR])
fn parse_range(pair: Pair<Rule>) -> Result<Range, YsetlParseError> {
    let mut parts = pair.into_inner();
    let start = parse_expr(careful_unwrap(parts.next())?)?;
    let op = careful_unwrap(parts.next())?;
    let end = parse_expr(careful_unwrap(parts.next())?)?;

    let inclusive = match op.as_rule() {
        Rule::inclusive_range_op => true,
        Rule::exclusive_range_op => false,
        _ => unreachable!(),
    };

    Ok(Range {
        inclusive,
        start: Box::new(start),
        end: Box::new(end),
        step: None,
    })
}

// interval_range_former([EXPR, RANGE_FORMER])
fn parse_interval_range_former(pair: Pair<Rule>) -> FormerResult {
    let mut parts = pair.into_inner();
    let step = parse_expr(careful_unwrap(parts.next())?)?;
    let mut range = parse_range(careful_unwrap(parts.next())?)?;
    range.step = Some(Box::new(step));
    Ok(Former::Range(range))
}

fn parse_iterator_former(pair: Pair<Rule>) -> FormerResult {
    let mut parts = pair.into_inner();
    let expr = parse_expr(careful_unwrap(parts.next())?)?;
    let iterator = parse_iterator(careful_unwrap(parts.next())?)?;
    Ok(Former::Iterator {
        output: Box::new(expr),
        iterator,
    })
}

fn parse_pre_op(op: Pair<Rule>, rhs: ExprResult) -> ExprResult {
    let op = match op.as_rule() {
        Rule::kw_not | Rule::bang => PreOp::Not,
        Rule::dash_pre => PreOp::Negate,
        Rule::plus => PreOp::Identity,
        Rule::hash => PreOp::Size,
        Rule::caret_pre => PreOp::Head,
        Rule::dollar => PreOp::Last,
        Rule::tilde => PreOp::Tail,
        Rule::amp_pre => PreOp::Init,
        _ => unreachable!(),
    };
    Ok(Expr::Prefix {
        op,
        rhs: Box::new(rhs?),
    })
}

fn parse_postfix(lhs: ExprResult, postfix: Pair<Rule>) -> ExprResult {
    let postfix = match postfix.as_rule() {
        Rule::fn_call => parse_fn_call(postfix)?,
        Rule::index_call => parse_index_call(postfix)?,
        Rule::pick_call => parse_pick_call(postfix)?,
        Rule::slice_call => parse_range_call(postfix)?,
        _ => unreachable!(),
    };
    Ok(Expr::Postfix {
        lhs: Box::new(lhs?),
        postfix,
    })
}

// fn_call([])
// fn_call([EXPR_LIST])
fn parse_fn_call(postfix: Pair<Rule>) -> PostfixResult {
    Ok(Postfix::Call(match postfix.into_inner().next() {
        Some(expr_list) => parse_expr_list(expr_list)?,
        None => Vec::new(),
    }))
}

// index_call([EXPR])
fn parse_index_call(postfix: Pair<Rule>) -> PostfixResult {
    let expr = parse_expr(careful_unwrap(postfix.into_inner().next())?)?;
    Ok(Postfix::Index(Box::new(expr)))
}

// pick_call([EXPR])
fn parse_pick_call(postfix: Pair<Rule>) -> PostfixResult {
    let expr = parse_expr(careful_unwrap(postfix.into_inner().next())?)?;
    Ok(Postfix::Pick(Box::new(expr)))
}

// range_call([EXPR, RANGE_OP, EXPR])
// range_call([EXPR, RANGE_OP])
// range_call([RANGE_OP, EXPR])
// range_call([RANGE_OP])
fn parse_range_call(postfix: Pair<Rule>) -> PostfixResult {
    let mut inclusive = true;
    let mut processed_op = false;
    let mut start: Option<Box<Expr>> = None;
    let mut end: Option<Box<Expr>> = None;

    // Kinda the ugliest code in here for me, but I couldn't find a more elegent
    // grammar for the optional expressions that would have made this prettier.
    for part in postfix.into_inner() {
        match part.as_rule() {
            Rule::inclusive_range_op => {
                processed_op = true;
            }
            Rule::exclusive_range_op => {
                processed_op = true;
                inclusive = false;
            }
            _ => {
                let expr = Some(Box::new(parse_expr(part)?));
                if processed_op {
                    end = expr;
                } else {
                    start = expr;
                }
            }
        }
    }
    Ok(Postfix::Slice {
        inclusive,
        start,
        end,
    })
}

fn parse_infix(lhs: ExprResult, op: Pair<Rule>, rhs: ExprResult) -> ExprResult {
    let op = match op.as_rule() {
        Rule::dbl_qst => BinOp::Nullcoel,
        Rule::dbl_star => BinOp::Exp,
        Rule::dbl_lt | Rule::kw_with => BinOp::WithBitLeft,
        Rule::dbl_gt | Rule::kw_less => BinOp::LessBitRight,
        Rule::dbl_eq | Rule::kw_iff => BinOp::Eq,
        Rule::dbl_amp | Rule::kw_and => BinOp::And,
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
        Rule::kw_mod | Rule::percent => BinOp::Mod,
        Rule::kw_in => BinOp::In,
        Rule::kw_notin => BinOp::Notin,
        Rule::kw_subset => BinOp::Subset,
        Rule::reduce_op => {
            let inner_op = careful_unwrap(op.into_inner().next())?;
            return Ok(match inner_op.as_rule() {
                Rule::ident => Expr::ReduceExpr {
                    reducer: Box::new(parse_ident(inner_op)?),
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                },
                Rule::nested_expr => Expr::ReduceExpr {
                    reducer: Box::new(parse_nested_expr(inner_op)?),
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                },
                _ => {
                    if let Some(binop) = parse_reducible_op(inner_op) {
                        Expr::ReduceOp {
                            op: binop,
                            lhs: Box::new(lhs?),
                            rhs: Box::new(rhs?),
                        }
                    } else {
                        Err(String::from("Reduce expression can only accept identifiers, nested expressions, or some binary operators"))?
                    }
                }
            });
        }
        Rule::infix_inject_op => {
            let inner_op = careful_unwrap(op.into_inner().next())?;
            println!("Parsing injection: {inner_op:?}");
            return Ok(match inner_op.as_rule() {
                Rule::ident => Expr::Inject {
                    injector: Box::new(parse_ident(inner_op)?),
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                },
                Rule::nested_expr => Expr::Inject {
                    injector: Box::new(parse_nested_expr(inner_op)?),
                    lhs: Box::new(lhs?),
                    rhs: Box::new(rhs?),
                },
                _ => Err(String::from(
                    "Inject expression can only accept identifiers and nested expressions.",
                ))?,
            });
        }
        _ => unreachable!(),
    };
    Ok(Expr::Infix {
        op,
        lhs: Box::new(lhs?),
        rhs: Box::new(rhs?),
    })
}

fn parse_bound(pair: Pair<Rule>) -> Bound {
    match pair.as_rule() {
        Rule::tilde => Bound::Tilde,
        Rule::ident => Bound::Ident(pair.as_str().to_owned()),
        Rule::bound_list => Bound::List(parse_bound_list(pair)),

        // bound_rest([])
        // bound_rest([IDENT])
        Rule::bound_rest => pair.into_inner().next().map_or(Bound::Rest, |inner| {
            Bound::RestOver(inner.as_str().to_owned())
        }),
        _ => unreachable!(),
    }
}

// bound_list([BOUND, BOUND, ..., BOUND])
fn parse_bound_list(pair: Pair<Rule>) -> BoundList {
    pair.into_inner().map(parse_bound).collect()
}

//  iterator([ITERATOR_LIST, EXPR?])
fn parse_iterator(pair: Pair<Rule>) -> Result<Iterator, YsetlParseError> {
    let mut parts = pair.into_inner();
    let iterators = parse_iterator_list(careful_unwrap(parts.next())?)?;
    let filter = match parts.next() {
        Some(expr) => Some(Box::new(parse_expr(expr)?)),
        None => None,
    };
    Ok(Iterator { iterators, filter })
}

// iterator_list([SINGLE_ITERATOR, SINGLE_ITERATOR, ..., SINGLE_ITERATOR])
fn parse_iterator_list(pair: Pair<Rule>) -> Result<IteratorList, YsetlParseError> {
    pair.into_inner().map(parse_single_iterator).collect()
}

fn parse_single_iterator(pair: Pair<Rule>) -> SingleIteratorResult {
    match pair.as_rule() {
        Rule::in_iterator => parse_in_iterator(pair),
        Rule::select_iterator_single => parse_select_iterator(pair),
        _ => unreachable!(),
    }
}

// in_iterator([BOUND_LIST, EXPR])
fn parse_in_iterator(pair: Pair<Rule>) -> SingleIteratorResult {
    let mut parts = pair.into_inner();
    let bound_list = parse_bound_list(careful_unwrap(parts.next())?);
    let expr = parse_expr(careful_unwrap(parts.next())?)?;
    Ok(SingleIterator::In {
        bounds: bound_list,
        expr,
    })
}

// select_iterator_single([BOUND, IDENT, BOUND])
fn parse_select_iterator(pair: Pair<Rule>) -> SingleIteratorResult {
    let mut parts = pair.into_inner();
    let value_bound = parse_bound(careful_unwrap(parts.next())?);
    let collection = careful_unwrap(parts.next())?.as_str().to_owned();
    let key_bound = parse_bound(careful_unwrap(parts.next())?);
    Ok(SingleIterator::Select {
        collection,
        key: key_bound,
        value: value_bound,
    })
}

// TODO: Try implementing using iterator.try_fold and std::ops::ControlFlow
// param_list([  ])
// param_list([ opt_param, ..., opt_param  ])
// param_list([ req_param, ..., req_param  ])
// param_list([ req_param, ..., req_param, opt_param, ..., opt_param  ])
// param_list([ ..., opt_param, req_param, ... ]) # VALID PARSE BUT INCORRECT
fn parse_param_list(param_list: Pair<Rule>) -> Result<ParamList, YsetlParseError> {
    let mut req = Vec::new();
    let mut opt = Vec::new();
    let mut parsing_opts = false;
    for param_rule in param_list.into_inner() {
        match param_rule.as_rule() {
            Rule::req_param => {
                if parsing_opts {
                    return Err("Required function params cannot follow optional params".to_owned());
                }
                req.push(param_rule.as_str().to_owned())
            }
            Rule::opt_param => {
                parsing_opts = true;
                opt.push(param_rule.as_str().replace("?", ""))
            }
            _ => unreachable!(),
        }
    }
    Ok((req, opt))
}

// func_literal([PARAM_LIST, EXPR])
fn parse_function_literal(expr: Pair<Rule>) -> ExprResult {
    let mut parts = expr.into_inner();
    let (req_params, opt_params) = parse_param_list(careful_unwrap(parts.next())?)?;
    let expr = parse_expr(careful_unwrap(parts.next())?)?;
    Ok(Expr::Function {
        req_params,
        opt_params,
        eval: Box::new(expr),
    })
}

// ternary_expr([EXPR, EXPR, EXPR])
fn parse_ternary_expr(expr: Pair<Rule>) -> ExprResult {
    let mut parts = expr.into_inner();
    let condition = Box::new(parse_expr(careful_unwrap(parts.next())?)?);
    let consequence = Box::new(parse_stmt(careful_unwrap(parts.next())?)?);
    let alternative = Box::new(parse_stmt(careful_unwrap(parts.next())?)?);
    Ok(Expr::Ternary {
        condition,
        consequence,
        alternative,
    })
}

// case([EXPR, EXPR])
// case([tilde, EXPR]) - The default case
fn parse_switch_case(case: Pair<Rule>) -> CaseResult {
    let mut parts = case.into_inner();
    let condition_pair = careful_unwrap(parts.next())?;
    let condition = match condition_pair.as_rule() {
        Rule::tilde => None,
        _ => Some(parse_expr(condition_pair)?),
    };
    let consequence = parse_stmt(careful_unwrap(parts.next())?)?;
    Ok(SwitchCase {
        condition,
        consequence,
    })
}

// switch_expr([CASE, CASE, ..., CASE])
// switch_expr([NESTED_EXPR, CASE, CASE, ..., CASE])
fn parse_switch_expr(expr: Pair<Rule>) -> ExprResult {
    let mut switch_condition: Option<Box<Expr>> = None;
    let mut cases: Vec<SwitchCase> = vec![];

    for part in expr.into_inner() {
        match part.as_rule() {
            Rule::nested_expr => {
                switch_condition = Some(Box::new(parse_nested_expr(part)?));
            }
            Rule::case => {
                cases.push(parse_switch_case(part)?);
            }
            _ => unreachable!(),
        };
    }

    Ok(Expr::Switch {
        condition: switch_condition,
        cases,
    })
}

// select_expr(["exists", ITERATOR])
// select_expr(["choose", ITERATOR])
// select_expr(["forall", ITERATOR])
fn parse_select_expr(expr: Pair<Rule>) -> ExprResult {
    let mut parts = expr.into_inner();
    let keyword = careful_unwrap(parts.next())?;
    let op = match keyword.as_rule() {
        Rule::kw_exists => SelectOp::EXISTS,
        Rule::kw_choose => SelectOp::CHOOSE,
        Rule::kw_forall => SelectOp::FORALL,
        _ => unreachable!(),
    };
    let next_part = careful_unwrap(parts.next())?;
    let iterator = parse_iterator(next_part)?;
    Ok(Expr::Select { op, iterator })
}

// block_expr([ STMT_LIST_WITH_CAPTURE ])
fn parse_block_expr(expr: Pair<Rule>) -> ExprResult {
    let inner = careful_unwrap(expr.into_inner().next())?;
    Ok(Expr::Block(parse_stmt_list_with_capture(inner)?))
}

// return_stmt([  ])
// return_stmt([ EXPR ])
fn parse_return_stmt(stmt: Pair<Rule>) -> StmtResult {
    stmt.into_inner()
        .next()
        .map(parse_expr)
        .map_or(Ok(Stmt::Return(None)), |v| {
            v.map(|expr| Stmt::Return(Some(expr)))
        })
}

// print_stmt([ EXPR ])
fn parse_print_stmt(stmt: Pair<Rule>) -> StmtResult {
    let inner = careful_unwrap(stmt.into_inner().next())?;
    let expr = parse_expr(inner)?;
    Ok(Stmt::Print(expr))
}

// assign_stmt([BOUND, EXPR])
fn parse_assign_stmt(stmt: Pair<Rule>) -> StmtResult {
    let mut parts = stmt.into_inner();
    let target = parse_bound(careful_unwrap(parts.next())?);
    let value = parse_expr(careful_unwrap(parts.next())?)?;
    Ok(Stmt::Assign { target, value })
}

// stmt_list_with_capture([ STMT_LIST, STMT? ])
fn parse_stmt_list_with_capture(pair: Pair<Rule>) -> BlockResult {
    let mut parts = pair.into_inner();
    let stmt_list = parse_stmt_list(careful_unwrap(parts.next())?)?;
    let implicit_return = parts
        .next()
        .map(parse_stmt)
        .map_or(Ok(None), |v| v.map(|stmt| Some(Box::new(stmt))))?;
    Ok(StmtListWithCapture {
        stmt_list,
        implicit_return,
    })
}

// stmt_list([  ])
// stmt_list([ STMT, STMT, ..., STMT ])
fn parse_stmt_list(stmt_list: Pair<Rule>) -> StmtListResult {
    stmt_list.into_inner().map(parse_stmt).collect()
}

fn span_start_str(span: Span) -> String {
    let (line, col) = span.start_pos().line_col();
    format!("Line {}, Col: {}", line, col)
}

// Only a subset of binary operators can be used in a reduce operation (and the subset
// operator ain't one of 'em). As long as the operator's first operand is the same type
// as the output, the operator can reduce over a collection, which is generally true
// for all the binary operations here. Technically, the arithmetic operators can start
// with an int and output a float, but they're interoperable on the same operator, so
// it's fine.
fn parse_reducible_op(pair: Pair<Rule>) -> Option<BinOp> {
    Some(match pair.as_rule() {
        Rule::dbl_qst => BinOp::Nullcoel,
        Rule::dbl_star => BinOp::Exp,
        Rule::dbl_lt | Rule::kw_with => BinOp::WithBitLeft,
        Rule::dbl_gt | Rule::kw_less => BinOp::LessBitRight,
        Rule::dbl_amp | Rule::kw_and => BinOp::And,
        Rule::dbl_pipe | Rule::kw_or => BinOp::Or,
        Rule::star | Rule::kw_inter => BinOp::Mult,
        Rule::slash | Rule::kw_div => BinOp::Div,
        Rule::plus | Rule::kw_union => BinOp::Add,
        Rule::dash => BinOp::Subtract,
        Rule::amp => BinOp::BitAnd,
        Rule::pipe => BinOp::BitOr,
        Rule::caret => BinOp::BitXor,
        Rule::kw_impl => BinOp::Impl,
        Rule::kw_mod | Rule::percent => BinOp::Mod,
        _ => None?,
    })
}

fn careful_unwrap(part: Option<Pair<Rule>>) -> Result<Pair<Rule>, YsetlParseError> {
    part.map_or_else(
        || {
            Err(String::from(
                "Something horrific and unexpected has occured, please consult your doctor.",
            ))
        },
        |part| Ok(part),
    )
}

use std::fmt::{Display, Formatter};
use super::grammar::Rule;
use pest::iterators::{Pair, Pairs};

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Rule::kw_null => write!(f, "kw_null"),
            Rule::kw_true => write!(f, "kw_true"),
            Rule::kw_false => write!(f, "kw_false"),
            Rule::kw_newat => write!(f, "kw_newat"),
            Rule::kw_fn => write!(f, "kw_fn"),
            Rule::kw_if => write!(f, "kw_if"),
            Rule::kw_case => write!(f, "kw_case"),
            Rule::kw_return => write!(f, "kw_return"),
            Rule::kw_not => write!(f, "kw_not"),
            Rule::kw_mod => write!(f, "kw_mod"),
            Rule::kw_in => write!(f, "kw_in"),
            Rule::kw_notin => write!(f, "kw_notin"),
            Rule::kw_subset => write!(f, "kw_subset"),
            Rule::kw_impl => write!(f, "kw_impl"),
            Rule::kw_iff => write!(f, "kw_iff"),
            Rule::kw_and => write!(f, "kw_and"),
            Rule::kw_or => write!(f, "kw_or"),
            Rule::kw_union => write!(f, "kw_union"),
            Rule::kw_inter => write!(f, "kw_inter"),
            Rule::kw_div => write!(f, "kw_div"),
            Rule::kw_with => write!(f, "kw_with"),
            Rule::kw_less => write!(f, "kw_less"),
            Rule::kw_exists => write!(f, "kw_exists"),
            Rule::kw_forall => write!(f, "kw_forall"),
            Rule::kw_choose => write!(f, "kw_choose"),
            Rule::kw_where => write!(f, "kw_where"),

            Rule::plus => write!(f, "plus"),
            Rule::dash => write!(f, "dash"),
            Rule::star => write!(f, "star"),
            Rule::slash => write!(f, "slash"),
            Rule::bang => write!(f, "bang"),
            Rule::hash => write!(f, "hash"),
            Rule::caret => write!(f, "caret"),
            Rule::dollar => write!(f, "dollar"),
            Rule::tilde => write!(f, "tilde"),
            Rule::amp => write!(f, "amp"),
            Rule::at => write!(f, "at"),
            Rule::lt => write!(f, "lt"),
            Rule::gt => write!(f, "gt"),
            Rule::dbl_eq => write!(f, "dbl_eq"),
            Rule::bang_eq => write!(f, "bang_eq"),
            Rule::lt_eq => write!(f, "lt_eq"),
            Rule::gt_eq => write!(f, "gt_eq"),
            Rule::dbl_star => write!(f, "dbl_star"),
            Rule::dbl_amp => write!(f, "dbl_amp"),
            Rule::dbl_pipe => write!(f, "dbl_pipe"),
            Rule::dbl_qst => write!(f, "dbl_qst"),
            Rule::dbl_lt => write!(f, "dbl_lt"),
            Rule::dbl_gt => write!(f, "dbl_gt"),

            Rule::nested_expression => write!(f, "nested_expression"),
            Rule::atom_keep => write!(f, "atom_keep"),
            Rule::atom => write!(f, "atom"),
            Rule::number_base => write!(f, "number_base"),
            Rule::number_decimal => write!(f, "number_decimal"),
            Rule::number_exp => write!(f, "number_exp"),
            Rule::number => write!(f, "number"),
            Rule::string_keep => write!(f, "string_keep"),
            Rule::string => write!(f, "string"),
            Rule::ident => write!(f, "ident"),
            Rule::tuple_literal => write!(f, "tuple_literal"),
            Rule::set_literal => write!(f, "set_literal"),

            Rule::iterator_former => write!(f, "iterator_former"),
            Rule::inclusive_range_op => write!(f, "inclusive_range_op"),
            Rule::exclusive_range_op => write!(f, "exclusive_range_op"),
            Rule::range_former => write!(f, "range_former"),
            Rule::interval_range_former => write!(f, "interval_range_former"),

            Rule::bound_list => write!(f, "bound_list"),
            Rule::in_iterator => write!(f, "in_iterator"),
            Rule::select_iterator_single => write!(f, "select_iterator_single"),
            Rule::select_iterator_multi => write!(f, "select_iterator_multi"),
            Rule::iterator_list => write!(f, "iterator_list"),
            Rule::iterator => write!(f, "iterator"),

            Rule::gen_expr => write!(f, "gen_expr"),
            Rule::expr => write!(f, "expr"),
            Rule::expr_list => write!(f, "expr_list"),
            rule => panic!("No display for rule: {:?}", rule),
        }
    }
}

pub fn pair_structure(pair: Pair<Rule>) -> String {
    let rule = pair.as_rule();
    let inner = pairs_structure(pair.into_inner());

    format!("{rule}{inner}")
}

pub fn pairs_structure(inner: Pairs<Rule>) -> String {
    if inner.len() == 0 {
        return format!("");
    }
    let joined_string = inner.map(pair_structure)
        .collect::<Vec<String>>()
        .join(", ");
    format!("([{joined_string}])")
}

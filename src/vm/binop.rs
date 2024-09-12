/**
 * This code was originally written in a more compact form, where the first-level match statements
 * checked the left and right side's types at the same time, and the operations between the same
 * types were batched together. In an attempt to both organize the typed operations a little better,
 * and theoretically produce better structures for optimization, I've changed the first-level match
 * to only check the operations, then from there determine the operations individually. This means
 * there is some duplication of match logic, but this is overall a lot easier for me to manage.
 */
use crate::object::object::{Object, ObjectOps, PreOps};
use crate::op::{self, lookup, Op};

pub fn execute_binop(op: Op, left: &Object, right: &Object) -> Result<Object, String> {
    match op {
        op::ADD => op_add(left, right),
        op::SUBTRACT => op_subtract(left, right),
        op::MULT => op_multiply(left, right),
        op::DIV => op_divide(left, right),
        op::MOD => op_modulus(left, right),
        op::EXP => op_exponentiation(left, right),
        op::LT => op_less_than(left, right),
        op::LTEQ => op_less_than_or_eq(left, right),
        op::GT => op_less_than(right, left),
        op::GTEQ => op_less_than_or_eq(right, left),
        op::TAKE => op_take(left, right),
        op::WITH_BIT_LEFT => op_with_bitshift_left(left, right),
        op::LESS_BIT_RIGHT => op_less_bitshift_right(left, right),
        op::BIT_AND => op_bitwise_and(left, right),
        op::BIT_OR => op_bitwise_or(left, right),
        op::BIT_XOR => op_bitwise_xor(left, right),
        op::LOGICAL_IMPL => op_logical_implication(left, right),
        op::IN => op_in(left, right),
        op::NOTIN => op_notin(left, right),
        op::SUBSET => op_subset(left, right),
        _ => {
            panic!("Tried to execute op {}", lookup(op).0);
        }
    }
}

fn op_error(op: &str, left: &Object, right: &Object) -> Result<Object, String> {
    Err(format!(
        "Cannot perform '{} {} {}'",
        left.to_debug_string(),
        op,
        right.to_debug_string()
    ))
}

fn op_add(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        // Adding numbers
        (Object::Int(left), Object::Int(right)) => Ok(Object::Int(left + right)),
        (Object::Float(left), Object::Float(right)) => Ok(Object::Float(left + right)),
        (Object::Int(left), Object::Float(right)) => Ok(Object::Float(*left as f64 + right)),
        (Object::Float(left), Object::Int(right)) => Ok(Object::Float(left + *right as f64)),

        // Set union
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Ok(Object::new_set(
            elements.union(other).map(Object::clone).collect(),
        )),

        // Concat tuples
        (Object::Tuple { .. }, Object::Tuple { .. }) => Ok(Object::new_tuple(
            vec![left.inner_tuple(), right.inner_tuple()].concat(),
        )),

        // Concat strings
        (Object::String { value, .. }, Object::String { value: other, .. }) => {
            let mut new_str = value.to_owned();
            new_str.push_str(other);
            Ok(Object::new_string(new_str))
        }

        // Map right-merge
        (Object::Map { .. }, Object::Map { .. }) => Ok(Object::new_map(
            left.inner_map()
                .into_iter()
                .chain(right.inner_map())
                .collect(),
        )),

        _ => op_error("+", left, right),
    }
}

fn op_subtract(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        // Subtracting numbers
        (Object::Int(left), Object::Int(right)) => Ok(Object::Int(left - right)),
        (Object::Float(left), Object::Float(right)) => Ok(Object::Float(left - right)),
        (Object::Int(left), Object::Float(right)) => Ok(Object::Float(*left as f64 - right)),
        (Object::Float(left), Object::Int(right)) => Ok(Object::Float(left - *right as f64)),

        // Set difference
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Ok(Object::new_set(
            elements.difference(other).map(Object::clone).collect(),
        )),

        _ => op_error("-", left, right),
    }
}

fn op_multiply(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        // Multiplying numbers
        (Object::Int(left), Object::Int(right)) => Ok(Object::Int(left * right)),
        (Object::Float(left), Object::Float(right)) => Ok(Object::Float(left * right)),
        (Object::Int(left), Object::Float(right)) => Ok(Object::Float(*left as f64 * right)),
        (Object::Float(left), Object::Int(right)) => Ok(Object::Float(left * *right as f64)),

        // String repetition
        (Object::Int(x), Object::String { value, .. })
        | (Object::String { value, .. }, Object::Int(x)) => {
            Ok(Object::new_string(String::from(value.repeat(*x as usize))))
        }

        // Tuple repetition
        (Object::Int(x), Object::Tuple { elements, .. })
        | (Object::Tuple { elements, .. }, Object::Int(x)) => {
            let capacity = *x as usize * elements.len();
            let new_elements: Vec<_> = elements
                .iter()
                .cycle()
                .take(capacity)
                .map(Object::clone)
                .collect();
            Ok(Object::new_tuple(new_elements))
        }

        // Set intersection
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Ok(Object::new_set(
            elements.intersection(other).map(Object::clone).collect(),
        )),

        // Tuple zip
        (Object::Tuple { .. }, Object::Tuple { .. }) => {
            let zipped_elements: Vec<_> = left
                .inner_tuple()
                .into_iter()
                .zip(right.inner_tuple())
                .map(|(l, r)| Object::new_tuple(vec![l, r]))
                .collect();
            Ok(Object::new_tuple(zipped_elements))
        }

        _ => op_error("*", left, right),
    }
}

fn op_divide(left: &Object, right: &Object) -> Result<Object, String> {
    if right.is_zero() {
        return Err(format!("Divide by zero error"));
    }
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => Ok(Object::Int(left / right)),
        (Object::Float(left), Object::Float(right)) => Ok(Object::Float(left / right)),
        (Object::Int(left), Object::Float(right)) => Ok(Object::Float(*left as f64 / right)),
        (Object::Float(left), Object::Int(right)) => Ok(Object::Float(left / *right as f64)),

        _ => op_error("/", left, right),
    }
}

fn op_modulus(left: &Object, right: &Object) -> Result<Object, String> {
    if right.is_zero() {
        return Err(format!("Divide by zero error"));
    }
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => Ok(Object::Int(left % right)),
        (Object::Float(left), Object::Float(right)) => Ok(Object::Float(left % right)),
        (Object::Int(left), Object::Float(right)) => Ok(Object::Float(*left as f64 % right)),
        (Object::Float(left), Object::Int(right)) => Ok(Object::Float(left % *right as f64)),

        _ => op_error("%", left, right),
    }
}

fn op_exponentiation(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => {
            if right.is_negative() {
                Ok(Object::Float((*left as f64).powf(*right as f64)))
            } else {
                Ok(Object::Int(left.pow(*right as u32)))
            }
        }
        (Object::Float(left), Object::Float(right)) => Ok(Object::Float(left.powf(*right))),
        (Object::Int(left), Object::Float(right)) => Ok(Object::Float((*left as f64).powf(*right))),
        (Object::Float(left), Object::Int(right)) => Ok(Object::Float(left.powf(*right as f64))),

        _ => op_error("**", left, right),
    }
}

fn op_less_than(left: &Object, right: &Object) -> Result<Object, String> {
    let left = left.ord_flt();
    let right = right.ord_flt();
    Ok(Object::Bool(left < right))
}

fn op_less_than_or_eq(left: &Object, right: &Object) -> Result<Object, String> {
    let left = left.ord_flt();
    let right = right.ord_flt();
    Ok(Object::Bool(left <= right))
}

fn op_take(left: &Object, right: &Object) -> Result<Object, String> {
    let left_value = match left {
        Object::Int(x) => *x,
        _ => return Err(format!("Left side of take operator must be an integer")),
    };
    let magnitude: usize = left_value.abs() as usize;
    let get_range = |col_size: usize| {
        if magnitude > col_size {
            std::ops::Range {
                start: 0,
                end: col_size,
            }
        } else if left_value.is_negative() {
            std::ops::Range {
                start: col_size - magnitude,
                end: col_size,
            }
        } else {
            std::ops::Range {
                start: 0,
                end: magnitude,
            }
        }
    };

    match right {
        Object::String { value, .. } => {
            let range = get_range(value.len());
            Ok(Object::new_string(value[range].to_owned()))
        }
        Object::Tuple { elements, .. } => {
            let range = get_range(elements.len());
            Ok(Object::new_tuple(
                elements[range].into_iter().map(Object::clone).collect(),
            ))
        }
        Object::Set { elements, .. } => {
            let new_elements: Vec<_> = elements.iter().take(magnitude).map(Object::clone).collect();
            Ok(Object::new_set_from_vec(new_elements))
        }
        _ => op_error("@", left, right),
    }
}

fn op_in(left: &Object, right: &Object) -> Result<Object, String> {
    match right {
        Object::Tuple { elements, .. } => Ok(Object::Bool(elements.contains(left))),
        Object::Set { elements, .. } => Ok(Object::Bool(elements.contains(left))),
        Object::String { value, .. } => {
            let char = match left {
                Object::String { value, .. } => {
                    if value.len() == 1 {
                        value.chars().next().unwrap()
                    } else {
                        return Err(format!(
                            "Can only check for membership of single char in a string"
                        ));
                    }
                }
                _ => {
                    return Err(format!(
                        "Cannot check for membership of {} in a string",
                        left.to_debug_string()
                    ))
                }
            };
            Ok(Object::Bool(value.contains(char)))
        }
        _ => op_error("in", left, right),
    }
}

fn op_notin(left: &Object, right: &Object) -> Result<Object, String> {
    if let Ok(in_result) = op_in(left, right) {
        Ok(in_result.not())
    } else {
        op_error("notin", left, right)
    }
}

fn op_subset(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Ok(Object::Bool(elements.is_subset(other))),
        _ => op_error("subset", left, right),
    }
}

fn op_bitwise_and(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Bool(x), Object::Bool(y)) => Ok(Object::Bool(x & y)),
        (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x & y)),

        // Set intersection
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Ok(Object::new_set(
            elements.intersection(other).map(Object::clone).collect(),
        )),

        // Map right-merge
        (Object::Map { .. }, Object::Map { .. }) => Ok(Object::new_map(
            left.inner_map()
                .into_iter()
                .chain(right.inner_map())
                .collect(),
        )),

        _ => op_error("&", left, right),
    }
}

fn op_bitwise_or(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Bool(x), Object::Bool(y)) => Ok(Object::Bool(x | y)),
        (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x | y)),

        // Set union
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Ok(Object::new_set(
            elements.union(other).map(Object::clone).collect(),
        )),
        _ => op_error("|", left, right),
    }
}

fn op_bitwise_xor(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Bool(x), Object::Bool(y)) => Ok(Object::Bool(x ^ y)),
        (Object::Int(x), Object::Int(y)) => Ok(Object::Int(x ^ y)),
        _ => op_error("^", left, right),
    }
}

fn op_logical_implication(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Bool(x), Object::Bool(y)) => Ok(Object::Bool((!x) | y)),
        _ => op_error("impl", left, right),
    }
}

fn op_with_bitshift_left(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Int(x), Object::Int(y)) => {
            if x.is_negative() || y.is_negative() {
                Err(format!(
                    "bitshift operations can't operate on negative integers"
                ))
            } else {
                Ok(Object::Int(x << y))
            }
        }
        (Object::Tuple { .. }, _) => {
            let mut new_elements = left.inner_tuple();
            new_elements.push(right.clone());
            Ok(Object::new_tuple(new_elements))
        }
        (Object::Set { .. }, _) => {
            let mut new_elements = left.inner_set();
            new_elements.insert(right.clone());
            Ok(Object::new_set(new_elements))
        }
        _ => op_error("<<", left, right),
    }
}

fn op_less_bitshift_right(left: &Object, right: &Object) -> Result<Object, String> {
    match (left, right) {
        (Object::Int(x), Object::Int(y)) => {
            if x.is_negative() || y.is_negative() {
                Err(format!(
                    "bitshift operations can't operate on negative integers"
                ))
            } else {
                Ok(Object::Int(x >> y))
            }
        }
        (Object::Tuple { .. }, _) => {
            let mut new_elements = left.inner_tuple();
            new_elements.insert(0, right.clone());
            Ok(Object::new_tuple(new_elements))
        }
        (Object::Set { .. }, _) => {
            let mut new_elements = left.inner_set();
            new_elements.remove(right);
            Ok(Object::new_set(new_elements))
        }
        (Object::Map { .. }, _) => {
            let mut new_elements = left.inner_map();
            new_elements.remove(right);
            Ok(Object::new_map(new_elements))
        }
        _ => op_error(">>", left, right),
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{object::object::{Atom, Object, PreOps}, op::{self, Op}};
    use super::execute_binop;
    type TestCase<'a> = (&'a Object, &'a Object, &'a Object);
    type TestCases<'a> = Vec<TestCase<'a>>;

    const ZERO_INT: Object = Object::Int(0);
    const THREE_INT: Object = Object::Int(3);
    const FIVE_INT: Object = Object::Int(5);
    const EIGHT_INT: Object = Object::Int(8);

    const THREE_FLOAT: Object = Object::Float(3.0);
    const FIVE_FLOAT: Object = Object::Float(5.0);
    const EIGHT_FLOAT: Object = Object::Float(8.0);

    fn assert_case(op: Op, l: &Object, r: &Object, o: &Object) {
        assert_eq!(execute_binop(op, &l, &r).as_ref(), Ok(o));
    }

    fn assert_cases(op: Op, cases: TestCases) {
        cases.into_iter().for_each(|(l, r, o)| {
            assert_case(op, &l, &r, o);
        });
    }

    fn make_tup(elements: &[i64]) -> Object {
        Object::new_tuple(elements.into_iter().map(|i| Object::Int(*i)).collect())
    }

    fn make_raw_set(elements: &[i64]) -> HashSet<Object> {
        elements.into_iter().map(|i| Object::Int(*i)).collect()
    }

    fn make_set(elements: &[i64]) -> Object {
        Object::new_set(make_raw_set(elements))
    }

    fn make_str(val: &str) -> Object {
        Object::new_string(val.to_owned())
    }

    fn make_atom(name: &str, i: u32) -> Object {
        Object::Atom(Atom::new(i, name.to_string()))
    }

    fn make_map(pairs: Vec<(&Object, i64)>) -> Object {
        let elements: HashMap<Object, Object> = pairs.into_iter()
            .fold(HashMap::new(), |mut map, (key, value)| {
                map.insert(key.clone(), Object::Int(value));
                return map
            });
        Object::new_map(elements)
    }

    #[test]
    fn test_addition() {
        assert_cases(op::ADD, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Int(13)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Float(13.0)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Float(13.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Float(13.0)),
        ]);
    }

    #[test]
    fn test_concatenation() {
        assert_cases(op::ADD, vec![
            (&make_tup(&[1,2,3]), &make_tup(&[4,5,6]), &make_tup(&[1,2,3,4,5,6])),
            (&make_str("abc"), &make_str("def"), &make_str("abcdef")),
        ]);
    }

    #[test]
    fn test_union() {
        assert_case(op::ADD, &make_set(&[1,2,3]), &make_set(&[2,3,4]), &make_set(&[1,2,3,4]));
        assert_case(op::BIT_OR, &make_set(&[1,2,3]), &make_set(&[2,3,4]), &make_set(&[1,2,3,4]));
    }

    #[test]
    fn test_right_merge() {
        let key_a = make_atom("a", 1);
        let key_b = make_atom("b", 2);
        let key_c = make_atom("c", 3);
        let key_d = make_atom("d", 4);

        let left_map = make_map(vec![(&key_a, 10), (&key_b, 20), (&key_c, 30)]);
        let right_map = make_map(vec![(&key_b, 99), (&key_d, 40)]);
        let result_map = make_map(vec![(&key_a, 10), (&key_b, 99), (&key_c, 30), (&key_d, 40)]);
        
        assert_case(op::ADD, &left_map, &right_map, &result_map);
        assert_case(op::BIT_AND, &left_map, &right_map, &result_map);
    }

    #[test]
    fn test_difference() {
        assert_cases(op::SUBTRACT, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Int(3)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Float(3.0)),
            (&make_set(&[1,2,3,4,5]), &make_set(&[2,3,4]), &make_set(&[1,5])),
        ]);
    }

    #[test]
    fn test_multiplication() {
        assert_cases(op::MULT, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Int(40)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Float(40.0)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Float(40.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Float(40.0)),
        ]);
    }

    #[test]
    fn test_intersection() {
        assert_case(op::MULT, &make_set(&[1,2,3]), &make_set(&[2,3,4]), &make_set(&[2,3]));
        assert_case(op::BIT_AND, &make_set(&[1,2,3]), &make_set(&[2,3,4]), &make_set(&[2,3]));
    }

    #[test]
    fn test_repetition() {
        assert_cases(op::MULT, vec![
            (&ZERO_INT, &make_tup(&[1,3,5]), &make_tup(&[])),
            (&THREE_INT, &make_tup(&[1,3,5]), &make_tup(&[1,3,5,1,3,5,1,3,5])),
            (&ZERO_INT, &make_str("abc"), &make_str("")),
            (&THREE_INT, &make_str("abc"), &make_str("abcabcabc")),
        ]);
    }

    #[test]
    fn test_zipping() {
        assert_case(op::MULT, &make_tup(&[1,2,3]), &make_tup(&[6,7,8,9]), &Object::new_tuple(vec![
            make_tup(&[1,6]),
            make_tup(&[2,7]),
            make_tup(&[3,8]),
        ]));
    }

    #[test]
    fn test_division() {
        assert_cases(op::DIV, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Int(1)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Float(1.6)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Float(1.6)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Float(1.6)),
        ]);
    }

    #[test]
    #[should_panic]
    fn test_division_divide_by_zero() {
        assert_case(op::DIV, &FIVE_INT, &ZERO_INT, &Object::Int(0));
    }

    #[test]
    fn test_modulus() {
        assert_cases(op::MOD, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Int(3)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Float(3.0)),
        ]);
    }

    #[test]
    #[should_panic]
    fn test_division_mod_by_zero() {
        assert_case(op::MOD, &FIVE_INT, &ZERO_INT, &Object::Int(0));
    }

    #[test]
    fn test_exponentiation() {
        assert_cases(op::EXP, vec![
            (&FIVE_INT, &THREE_INT, &Object::Int(125)),
            (&FIVE_INT, &THREE_FLOAT, &Object::Float(125.0)),
            (&FIVE_FLOAT, &THREE_INT, &Object::Float(125.0)),
            (&FIVE_FLOAT, &THREE_FLOAT, &Object::Float(125.0)),

            (&FIVE_INT, &THREE_INT.negate(), &Object::Float(0.008)),
            (&FIVE_INT, &THREE_FLOAT.negate(), &Object::Float(0.008)),
            (&FIVE_FLOAT, &THREE_INT.negate(), &Object::Float(0.008)),
            (&FIVE_FLOAT, &THREE_FLOAT.negate(), &Object::Float(0.008)),
        ]);
    }

    #[test]
    fn test_less_than() {
        assert_cases(op::LT, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Bool(false)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Bool(false)),
            (&FIVE_INT, &EIGHT_INT, &Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_INT, &Object::Bool(true)),
            (&FIVE_INT, &EIGHT_FLOAT, &Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, &Object::Bool(true)),
            (&FIVE_INT, &FIVE_INT, &Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_INT, &Object::Bool(false)),
            (&FIVE_INT, &FIVE_FLOAT, &Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_FLOAT, &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_less_than_or_equal() {
        assert_cases(op::LTEQ, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Bool(false)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Bool(false)),
            (&FIVE_INT, &EIGHT_INT, &Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_INT, &Object::Bool(true)),
            (&FIVE_INT, &EIGHT_FLOAT, &Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, &Object::Bool(true)),
            (&FIVE_INT, &FIVE_INT, &Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_INT, &Object::Bool(true)),
            (&FIVE_INT, &FIVE_FLOAT, &Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_FLOAT, &Object::Bool(true)),
        ]);
    }

    #[test]
    fn test_greater_than() {
        assert_cases(op::GT, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Bool(true)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Bool(true)),
            (&FIVE_INT, &EIGHT_INT, &Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_INT, &Object::Bool(false)),
            (&FIVE_INT, &EIGHT_FLOAT, &Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, &Object::Bool(false)),
            (&FIVE_INT, &FIVE_INT, &Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_INT, &Object::Bool(false)),
            (&FIVE_INT, &FIVE_FLOAT, &Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_FLOAT, &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_greater_than_or_equal() {
        assert_cases(op::GTEQ, vec![
            (&EIGHT_INT, &FIVE_INT, &Object::Bool(true)),
            (&EIGHT_INT, &FIVE_FLOAT, &Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_INT, &Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, &Object::Bool(true)),
            (&FIVE_INT, &EIGHT_INT, &Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_INT, &Object::Bool(false)),
            (&FIVE_INT, &EIGHT_FLOAT, &Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, &Object::Bool(false)),
            (&FIVE_INT, &FIVE_INT, &Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_INT, &Object::Bool(true)),
            (&FIVE_INT, &FIVE_FLOAT, &Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_FLOAT, &Object::Bool(true)),
        ]);
    }

    #[test]
    fn test_take_from_string() {
        let string_rhs = make_str("abcde");
        let string_empty = make_str("");

        assert_cases(op::TAKE, vec![
            (&THREE_INT, &string_rhs, &make_str("abc")),
            (&THREE_INT.negate(), &string_rhs, &make_str("cde")),

            (&ZERO_INT, &string_rhs, &string_empty),
            (&ZERO_INT.negate(), &string_rhs, &string_empty),

            (&FIVE_INT, &string_rhs, &string_rhs),
            (&FIVE_INT.negate(), &string_rhs, &string_rhs),

            (&EIGHT_INT, &string_rhs, &string_rhs),
            (&EIGHT_INT.negate(), &string_rhs, &string_rhs),
        ]);
    }

    #[test]
    fn test_take_from_tuple() {
        let tuple_rhs = make_tup(&[1,2,3,4,5]);
        let tuple_empty = make_tup(&[]);

        assert_cases(op::TAKE, vec![
            (&THREE_INT, &tuple_rhs, &make_tup(&[1,2,3])),
            (&THREE_INT.negate(), &tuple_rhs, &make_tup(&[3,4,5])),

            (&ZERO_INT, &tuple_rhs, &tuple_empty),
            (&ZERO_INT.negate(), &tuple_rhs, &tuple_empty),

            (&FIVE_INT, &tuple_rhs, &tuple_rhs),
            (&FIVE_INT.negate(), &tuple_rhs, &tuple_rhs),

            (&EIGHT_INT, &tuple_rhs, &tuple_rhs),
            (&EIGHT_INT.negate(), &tuple_rhs, &tuple_rhs),
        ]);
    }

    #[test]
    fn test_take_from_set() {
        let set_rhs = make_set(&[1,2,3,4,5]);
        let set_empty = make_set(&[]);

        assert_cases(op::TAKE, vec![
            (&ZERO_INT, &set_rhs, &set_empty),
            (&ZERO_INT.negate(), &set_rhs, &set_empty),

            (&FIVE_INT, &set_rhs, &set_rhs),
            (&FIVE_INT.negate(), &set_rhs, &set_rhs),

            (&EIGHT_INT, &set_rhs, &set_rhs),
            (&EIGHT_INT.negate(), &set_rhs, &set_rhs),
        ]);

        // Since there's no ordering inside of sets, we can only guarantee the size of the output

        let inner_orig = make_raw_set(&[1,2,3,4,5]);

        match execute_binop(op::TAKE, &THREE_INT, &set_rhs).unwrap() {
            Object::Set { elements, .. } => {
                assert_eq!(elements.len(), 3);
                assert!(elements.is_subset(&inner_orig));
            },
            _ => panic!("Did not evaluate to set"),
        };

        match execute_binop(op::TAKE, &THREE_INT.negate(), &set_rhs).unwrap() {
            Object::Set { elements, .. } => {
                assert_eq!(elements.len(), 3);
                assert!(elements.is_subset(&inner_orig));
            },
            _ => panic!("Did not evaluate to set"),
        };
    }

    #[test]
    fn test_membership() {
        assert_cases(op::IN, vec![
            (&THREE_INT, &make_tup(&[1,2,3]), &Object::Bool(true)),
            (&FIVE_INT, &make_tup(&[1,2,3]), &Object::Bool(false)),
            (&THREE_INT, &make_set(&[1,2,3]), &Object::Bool(true)),
            (&FIVE_INT, &make_set(&[1,2,3]), &Object::Bool(false)),
            (&make_str("a"), &make_str("abc"), &Object::Bool(true)),
            (&make_str("d"), &make_str("abc"), &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_non_membership() {
        assert_cases(op::NOTIN, vec![
            (&THREE_INT, &make_tup(&[1,2,3]), &Object::Bool(false)),
            (&FIVE_INT, &make_tup(&[1,2,3]), &Object::Bool(true)),
            (&THREE_INT, &make_set(&[1,2,3]), &Object::Bool(false)),
            (&FIVE_INT, &make_set(&[1,2,3]), &Object::Bool(true)),
            (&make_str("a"), &make_str("abc"), &Object::Bool(false)),
            (&make_str("d"), &make_str("abc"), &Object::Bool(true)),
        ]);
    }

    #[test]
    fn test_subset() {
        assert_cases(op::SUBSET, vec![
            (&make_set(&[1,3]), &make_set(&[1,2,3,4]), &Object::Bool(true)),
            (&make_set(&[1,5]), &make_set(&[1,2,3,4]), &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_bitwise_and() {
        assert_cases(op::BIT_AND, vec![
            (&Object::Int(0b10101010), &Object::Int(0b11110000), &Object::Int(0b10100000)),
            (&Object::Bool(true), &Object::Bool(true), &Object::Bool(true)),
            (&Object::Bool(true), &Object::Bool(false), &Object::Bool(false)),
            (&Object::Bool(false), &Object::Bool(true), &Object::Bool(false)),
            (&Object::Bool(false), &Object::Bool(false), &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_bitwise_or() {
        assert_cases(op::BIT_OR, vec![
            (&Object::Int(0b10101010), &Object::Int(0b11110000), &Object::Int(0b11111010)),
            (&Object::Bool(true), &Object::Bool(true), &Object::Bool(true)),
            (&Object::Bool(true), &Object::Bool(false), &Object::Bool(true)),
            (&Object::Bool(false), &Object::Bool(true), &Object::Bool(true)),
            (&Object::Bool(false), &Object::Bool(false), &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_bitwise_xor() {
        assert_cases(op::BIT_XOR, vec![
            (&Object::Int(0b10101010), &Object::Int(0b11110000), &Object::Int(0b01011010)),
            (&Object::Bool(true), &Object::Bool(true), &Object::Bool(false)),
            (&Object::Bool(true), &Object::Bool(false), &Object::Bool(true)),
            (&Object::Bool(false), &Object::Bool(true), &Object::Bool(true)),
            (&Object::Bool(false), &Object::Bool(false), &Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_implication() {
        assert_cases(op::LOGICAL_IMPL, vec![
            (&Object::Bool(true), &Object::Bool(true), &Object::Bool(true)),
            (&Object::Bool(true), &Object::Bool(false), &Object::Bool(false)),
            (&Object::Bool(false), &Object::Bool(true), &Object::Bool(true)),
            (&Object::Bool(false), &Object::Bool(false), &Object::Bool(true)),
        ]);
    }

    #[test]
    fn test_bitshift_left() {
        assert_case(op::WITH_BIT_LEFT, &Object::Int(0b110), &Object::Int(3), &Object::Int(0b110000));
    }

    #[test]
    #[should_panic]
    fn test_bitshift_left_negative() {
        assert_case(op::WITH_BIT_LEFT, &Object::Int(0b110), &Object::Int(-3), &Object::Null);
    }

    #[test]
    fn test_tuple_push() {
        assert_case(op::WITH_BIT_LEFT, &make_tup(&[1,2,3]), &Object::Int(4), &make_tup(&[1,2,3,4]));
    }

    #[test]
    fn test_set_insert() {
        assert_case(op::WITH_BIT_LEFT, &make_set(&[1,2,3]), &Object::Int(4), &make_set(&[1,2,3,4]));
    }

    #[test]
    fn test_bitshift_right() {
        assert_case(op::LESS_BIT_RIGHT, &Object::Int(0b110000), &Object::Int(3), &Object::Int(0b110));
    }

    #[test]
    #[should_panic]
    fn test_bitshift_right_negative() {
        assert_case(op::LESS_BIT_RIGHT, &Object::Int(0b110000), &Object::Int(-3), &Object::Null);
    }

    #[test]
    fn test_tuple_unshift() {
        assert_case(op::LESS_BIT_RIGHT, &make_tup(&[2,3,4]), &Object::Int(1), &make_tup(&[1,2,3,4]));
    }

    #[test]
    fn test_map_less() {
        let key_a = make_atom("a", 1);
        let key_b = make_atom("b", 2);
        assert_case(
            op::LESS_BIT_RIGHT,
            &make_map(vec![(&key_a, 10), (&key_b, 20)]),
            &key_a,
            &make_map(vec![(&key_b, 20)]),
        );
    }

    #[test]
    fn test_set_less() {
        assert_case(op::LESS_BIT_RIGHT, &make_set(&[1,2,3]), &Object::Int(2), &make_set(&[1,3]));
    }
}

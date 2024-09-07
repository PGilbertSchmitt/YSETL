/**
 * This code was originally written in a more compact form, where the first-level match statements
 * checked the left and right side's types at the same time, and the operations between the same
 * types were batched together. In an attempt to both organize the typed operations a little better,
 * and theoretically produce better structures for optimization, I've changed the first-level match
 * to only check the operations, then from there determine the operations individually. This means
 * there is some duplication of match logic, but this is overall a lot easier for me to manage.
 */
use crate::object::object::{Object, ObjectOps};
use crate::op::{self, Op};

pub fn execute_binop(op: Op, left: &Object, right: &Object) -> Object {
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
        op::TAKE => todo!(),
        op::WITH_BIT_LEFT => todo!(),
        op::LESS_BIT_RIGHT => todo!(),
        op::BIT_AND => todo!(),
        op::BIT_OR => todo!(),
        op::BIT_XOR => todo!(),
        op::IN => todo!(),
        op::NOTIN => todo!(),
        op::SUBSET => todo!(),
        op::LOGICAL_IMPL => todo!(),
        _ => unreachable!(),
    }
}

fn op_add(left: &Object, right: &Object) -> Object {
    match (left, right) {
        // Adding numbers
        (Object::Int(left), Object::Int(right)) => Object::Int(left + right),
        (Object::Float(left), Object::Float(right)) => Object::Float(left + right),
        (Object::Int(left), Object::Float(right)) => Object::Float(*left as f64 + right),
        (Object::Float(left), Object::Int(right)) => Object::Float(left + *right as f64),

        // Concat sets
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Object::new_set(elements.union(other).map(Object::clone).collect()),

        // Concat tuples
        (Object::Tuple { .. }, Object::Tuple { .. }) => {
            Object::new_tuple(vec![left.inner_tuple(), right.inner_tuple()].concat())
        }

        // Concat strings
        (Object::String { value, .. }, Object::String { value: other, .. }) => {
            let mut new_str = value.to_owned();
            new_str.push_str(other);
            Object::new_string(new_str)
        }

        _ => panic!(
            "Cannot add or union types {} and {}",
            left.to_debug_string(),
            right.to_debug_string()
        ),
    }
}

fn op_subtract(left: &Object, right: &Object) -> Object {
    match (left, right) {
        // Subtracting numbers
        (Object::Int(left), Object::Int(right)) => Object::Int(left - right),
        (Object::Float(left), Object::Float(right)) => Object::Float(left - right),
        (Object::Int(left), Object::Float(right)) => Object::Float(*left as f64 - right),
        (Object::Float(left), Object::Int(right)) => Object::Float(left - *right as f64),

        // Set difference
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Object::new_set(elements.difference(other).map(Object::clone).collect()),

        _ => panic!(
            "Cannot subtract types {} and {}",
            left.to_debug_string(),
            right.to_debug_string()
        ),
    }
}

fn op_multiply(left: &Object, right: &Object) -> Object {
    match (left, right) {
        // Multiplying numbers
        (Object::Int(left), Object::Int(right)) => Object::Int(left * right),
        (Object::Float(left), Object::Float(right)) => Object::Float(left * right),
        (Object::Int(left), Object::Float(right)) => Object::Float(*left as f64 * right),
        (Object::Float(left), Object::Int(right)) => Object::Float(left * *right as f64),

        // String repetition
        (Object::Int(x), Object::String { value, .. })
        | (Object::String { value, .. }, Object::Int(x)) => {
            Object::new_string(String::from(value.repeat(*x as usize)))
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
            Object::new_tuple(new_elements)
        }

        // Set intersection
        (
            Object::Set { elements, .. },
            Object::Set {
                elements: other, ..
            },
        ) => Object::new_set(elements.intersection(other).map(Object::clone).collect()),

        // Tuple zip
        (Object::Tuple { .. }, Object::Tuple { .. }) => {
            let zipped_elements: Vec<_> = left
                .inner_tuple()
                .into_iter()
                .zip(right.inner_tuple())
                .map(|(l, r)| Object::new_tuple(vec![l, r]))
                .collect();
            Object::new_tuple(zipped_elements)
        }

        _ => panic!(
            "Cannot multiply or intersect types {} and {}",
            left.to_debug_string(),
            right.to_debug_string()
        ),
    }
}

fn op_divide(left: &Object, right: &Object) -> Object {
    if right.is_zero() {
        panic!("Divide by zero error");
    }
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => Object::Int(left / right),
        (Object::Float(left), Object::Float(right)) => Object::Float(left / right),
        (Object::Int(left), Object::Float(right)) => Object::Float(*left as f64 / right),
        (Object::Float(left), Object::Int(right)) => Object::Float(left / *right as f64),

        _ => panic!(
            "Cannot divide types {} and {}",
            left.to_debug_string(),
            right.to_debug_string()
        ),
    }
}

fn op_modulus(left: &Object, right: &Object) -> Object {
    if right.is_zero() {
        panic!("Divide by zero error");
    }
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => Object::Int(left % right),
        (Object::Float(left), Object::Float(right)) => Object::Float(left % right),
        (Object::Int(left), Object::Float(right)) => Object::Float(*left as f64 % right),
        (Object::Float(left), Object::Int(right)) => Object::Float(left % *right as f64),

        _ => panic!(
            "Cannot divide types {} and {}",
            left.to_debug_string(),
            right.to_debug_string()
        ),
    }
}

fn op_exponentiation(left: &Object, right: &Object) -> Object {
    match (left, right) {
        (Object::Int(left), Object::Int(right)) => {
            if *right < 0 {
                Object::Float((*left as f64).powf(*right as f64))
            } else {
                Object::Int(left.pow(*right as u32))
            }
        }
        (Object::Float(left), Object::Float(right)) => Object::Float(left.powf(*right)),
        (Object::Int(left), Object::Float(right)) => Object::Float((*left as f64).powf(*right)),
        (Object::Float(left), Object::Int(right)) => Object::Float(left.powf(*right as f64)),

        _ => panic!(
            "Cannot divide types {} and {}",
            left.to_debug_string(),
            right.to_debug_string()
        ),
    }
}

fn op_less_than(left: &Object, right: &Object) -> Object {
    let left = left.ord_flt();
    let right = right.ord_flt();
    Object::Bool(left < right)
}

fn op_less_than_or_eq(left: &Object, right: &Object) -> Object {
    let left = left.ord_flt();
    let right = right.ord_flt();
    Object::Bool(left <= right)
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use crate::{object::object::{Object, PreOps}, op::{self, Op}};
    use super::execute_binop;
    type TestCase<'a> = (&'a Object, &'a Object, Object);
    type TestCases<'a> = Vec<TestCase<'a>>;

    const ZERO_INT: Object = Object::Int(0);
    const THREE_INT: Object = Object::Int(3);
    const FIVE_INT: Object = Object::Int(5);
    const EIGHT_INT: Object = Object::Int(8);

    const THREE_FLOAT: Object = Object::Float(3.0);
    const FIVE_FLOAT: Object = Object::Float(5.0);
    const EIGHT_FLOAT: Object = Object::Float(8.0);

    fn assert_case(op: Op, l: &Object, r: &Object, o: Object) {
        assert_eq!(execute_binop(op, &l, &r), o);
    }

    fn assert_cases(op: Op, cases: TestCases) {
        cases.into_iter().for_each(|(l, r, o)| {
            assert_case(op, &l, &r, o);
        });
    }

    fn make_tup(elements: &[i64]) -> Object {
        Object::new_tuple(elements.into_iter().map(|i| Object::Int(*i)).collect())
    }

    fn make_set(elements: &[i64]) -> Object {
        Object::new_set_from_vec(elements.into_iter().map(|i| Object::Int(*i)).collect())
    }

    fn make_str(val: &str) -> Object {
        Object::new_string(val.to_owned())
    }

    #[test]
    fn test_addition() {
        assert_cases(op::ADD, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Int(13)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Float(13.0)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Float(13.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Float(13.0)),
        ]);
    }

    #[test]
    fn test_concatenation() {
        assert_cases(op::ADD, vec![
            (&make_tup(&[1,2,3]), &make_tup(&[4,5,6]), make_tup(&[1,2,3,4,5,6])),
            (&make_str("abc"), &make_str("def"), make_str("abcdef")),
        ]);
    }

    #[test]
    fn test_union() {
        assert_case(op::ADD, &make_set(&[1,2,3]), &make_set(&[2,3,4]), make_set(&[1,2,3,4]));
    }

    #[test]
    fn test_difference() {
        assert_cases(op::SUBTRACT, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Int(3)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Float(3.0)),
            (&make_set(&[1,2,3,4,5]), &make_set(&[2,3,4]), make_set(&[1,5])),
        ]);
    }

    #[test]
    fn test_multiplication() {
        assert_cases(op::MULT, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Int(40)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Float(40.0)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Float(40.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Float(40.0)),
        ]);
    }

    #[test]
    fn test_intersection() {
        assert_case(op::MULT, &make_set(&[1,2,3]), &make_set(&[2,3,4]), make_set(&[2,3]));
    }

    #[test]
    fn test_repetition() {
        assert_cases(op::MULT, vec![
            (&ZERO_INT, &make_tup(&[1,3,5]), make_tup(&[])),
            (&THREE_INT, &make_tup(&[1,3,5]), make_tup(&[1,3,5,1,3,5,1,3,5])),
            (&ZERO_INT, &make_str("abc"), make_str("")),
            (&THREE_INT, &make_str("abc"), make_str("abcabcabc")),
        ]);
    }

    #[test]
    fn test_zipping() {
        assert_case(op::MULT, &make_tup(&[1,2,3]), &make_tup(&[6,7,8,9]), Object::new_tuple(vec![
            make_tup(&[1,6]),
            make_tup(&[2,7]),
            make_tup(&[3,8]),
        ]));
    }

    #[test]
    fn test_division() {
        assert_cases(op::DIV, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Int(1)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Float(1.6)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Float(1.6)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Float(1.6)),
        ]);
    }

    #[test]
    #[should_panic]
    fn test_division_divide_by_zero() {
        assert_case(op::DIV, &FIVE_INT, &ZERO_INT, Object::Int(0));
    }

    #[test]
    fn test_modulus() {
        assert_cases(op::MOD, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Int(3)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Float(3.0)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Float(3.0)),
        ]);
    }

    #[test]
    #[should_panic]
    fn test_division_mod_by_zero() {
        assert_case(op::MOD, &FIVE_INT, &ZERO_INT, Object::Int(0));
    }

    #[test]
    fn test_exponentiation() {
        assert_cases(op::EXP, vec![
            (&FIVE_INT, &THREE_INT, Object::Int(125)),
            (&FIVE_INT, &THREE_FLOAT, Object::Float(125.0)),
            (&FIVE_FLOAT, &THREE_INT, Object::Float(125.0)),
            (&FIVE_FLOAT, &THREE_FLOAT, Object::Float(125.0)),

            (&FIVE_INT, &THREE_INT.negate(), Object::Float(0.008)),
            (&FIVE_INT, &THREE_FLOAT.negate(), Object::Float(0.008)),
            (&FIVE_FLOAT, &THREE_INT.negate(), Object::Float(0.008)),
            (&FIVE_FLOAT, &THREE_FLOAT.negate(), Object::Float(0.008)),
        ]);
    }

    #[test]
    fn test_less_than() {
        assert_cases(op::LT, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Bool(false)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Bool(false)),
            (&FIVE_INT, &EIGHT_INT, Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_INT, Object::Bool(true)),
            (&FIVE_INT, &EIGHT_FLOAT, Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, Object::Bool(true)),
            (&FIVE_INT, &FIVE_INT, Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_INT, Object::Bool(false)),
            (&FIVE_INT, &FIVE_FLOAT, Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_FLOAT, Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_less_than_or_equal() {
        assert_cases(op::LTEQ, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Bool(false)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Bool(false)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Bool(false)),
            (&FIVE_INT, &EIGHT_INT, Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_INT, Object::Bool(true)),
            (&FIVE_INT, &EIGHT_FLOAT, Object::Bool(true)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, Object::Bool(true)),
            (&FIVE_INT, &FIVE_INT, Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_INT, Object::Bool(true)),
            (&FIVE_INT, &FIVE_FLOAT, Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_FLOAT, Object::Bool(true)),
        ]);
    }

    #[test]
    fn test_greater_than() {
        assert_cases(op::GT, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Bool(true)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Bool(true)),
            (&FIVE_INT, &EIGHT_INT, Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_INT, Object::Bool(false)),
            (&FIVE_INT, &EIGHT_FLOAT, Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, Object::Bool(false)),
            (&FIVE_INT, &FIVE_INT, Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_INT, Object::Bool(false)),
            (&FIVE_INT, &FIVE_FLOAT, Object::Bool(false)),
            (&FIVE_FLOAT, &FIVE_FLOAT, Object::Bool(false)),
        ]);
    }

    #[test]
    fn test_greater_than_or_equal() {
        assert_cases(op::GTEQ, vec![
            (&EIGHT_INT, &FIVE_INT, Object::Bool(true)),
            (&EIGHT_INT, &FIVE_FLOAT, Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_INT, Object::Bool(true)),
            (&EIGHT_FLOAT, &FIVE_FLOAT, Object::Bool(true)),
            (&FIVE_INT, &EIGHT_INT, Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_INT, Object::Bool(false)),
            (&FIVE_INT, &EIGHT_FLOAT, Object::Bool(false)),
            (&FIVE_FLOAT, &EIGHT_FLOAT, Object::Bool(false)),
            (&FIVE_INT, &FIVE_INT, Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_INT, Object::Bool(true)),
            (&FIVE_INT, &FIVE_FLOAT, Object::Bool(true)),
            (&FIVE_FLOAT, &FIVE_FLOAT, Object::Bool(true)),
        ]);
    }
}

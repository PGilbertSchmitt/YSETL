use crate::object::{BaseObject, Object, ObjectOps};
use crate::op::debug::lookup;
use crate::op::{self, Op};

pub fn from_bool(val: bool) -> BaseObject {
    if val { BaseObject::True } else { BaseObject::False }
}

pub fn execute_binop(op: Op, left: Object, right: Object) -> Object {
    match (left.as_ref(), right.as_ref()) {
        (&BaseObject::Int(left), &BaseObject::Int(right)) => {
            execute_int_math(op, left, right).wrap()
        },
        (&BaseObject::Float(left), &BaseObject::Float(right)) => {
            execute_float_math(op, left, right).wrap()
        }
        (&BaseObject::Int(left), &BaseObject::Float(right)) => {
            execute_float_math(op, left as f64, right).wrap()
        }
        (&BaseObject::Float(left), &BaseObject::Int(right)) => {
            execute_float_math(op, left, right as f64).wrap()
        }
        _ => panic!(
            "Could not perform op {} on types {} and {}",
            lookup(op).0,
            left.to_s(),
            right.to_s(),
        )
    }
}

fn execute_int_math(op: Op, left: i64, right: i64) -> BaseObject {
    match op {
        op::ADD => BaseObject::Int(left + right),
        op::SUBSET => BaseObject::Int(left - right),
        op::MULT => BaseObject::Int(left * right),
        op::DIV => {
            if right == 0 {
                panic!("Divide by zero error!");
            }
            BaseObject::Int(left / right)
        }
        op::MOD => {
            if right == 0 {
                panic!("Mod by zero error!");
            }
            BaseObject::Int(left % right)
        }
        op::LT => from_bool(left < right),
        op::LTEQ => from_bool(left <= right),
        op::EQ => from_bool(left == right),
        op::NEQ => from_bool(left != right),
        _ => unimplemented!(),
    }
}

// duplicate to int math for most operations, can that be consolidated or no?
fn execute_float_math(op: Op, left: f64, right: f64) -> BaseObject {
    match op {
        op::ADD => BaseObject::Float(left + right),
        op::SUBSET => BaseObject::Float(left - right),
        op::MULT => BaseObject::Float(left * right),
        op::DIV => {
            if right == 0.0 {
                panic!("Divide by zero error!");
            }
            BaseObject::Float(left / right)
        }
        op::MOD => {
            if right == 0.0 {
                panic!("Mod by zero error!");
            }
            BaseObject::Float(left % right)
        }
        op::LT => from_bool(left < right),
        op::LTEQ => from_bool(left <= right),
        op::EQ => from_bool(left == right),
        op::NEQ => from_bool(left != right),
        _ => unimplemented!(),
    }
}

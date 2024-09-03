use crate::object::object::{Object, ObjectOps};
use crate::op::lookup;
use crate::op::{self, Op};

pub fn execute_binop(op: Op, left: Object, right: Object) -> Object {
    match (&left, &right) {
        (&Object::Bool(left), &Object::Bool(right)) => execute_bool_math(op, left, right),

        (&Object::Int(left), &Object::Int(right)) => execute_int_math(op, left, right),
        (&Object::Float(left), &Object::Float(right)) => execute_float_math(op, left, right),
        (&Object::Int(left), &Object::Float(right)) => execute_float_math(op, left as f64, right),
        (&Object::Float(left), &Object::Int(right)) => execute_float_math(op, left, right as f64),

        _ => panic!(
            "Could not perform op {} on values {} and {}",
            lookup(op).0,
            left.to_debug_string(),
            right.to_debug_string(),
        )
    }
}

fn execute_int_math(op: Op, left: i64, right: i64) -> Object {
    match op {
        op::ADD => Object::Int(left + right),
        op::SUBTRACT => Object::Int(left - right),
        op::MULT => Object::Int(left * right),
        op::EXP => {
            if right < 0 {
                Object::Float((left as f64).powf(right as f64))
            } else {
                Object::Int(left.pow(right as u32))
            }
        }
        op::DIV => {
            if right == 0 {
                panic!("Divide by zero error!");
            }
            Object::Int(left / right)
        }
        op::MOD => {
            if right == 0 {
                panic!("Mod by zero error!");
            }
            Object::Int(left % right)
        }
        op::LT => Object::Bool(left < right),
        op::LTEQ => Object::Bool(left <= right),
        op::GT => Object::Bool(left > right),
        op::GTEQ => Object::Bool(left >= right),

        _ => panic!(
            "Could not perform op {} on integers i{} and i{}",
            lookup(op).0,
            left,
            right,
        ),
    }
}

// duplicate to int math for most operations, can that be consolidated or no?
fn execute_float_math(op: Op, left: f64, right: f64) -> Object {
    match op {
        op::ADD => Object::Float(left + right),
        op::SUBTRACT => Object::Float(left - right),
        op::MULT => Object::Float(left * right),
        op::EXP => Object::Float(left.powf(right)),
        op::DIV => {
            if right == 0.0 {
                panic!("Divide by zero error!");
            }
            Object::Float(left / right)
        }
        op::MOD => {
            if right == 0.0 {
                panic!("Mod by zero error!");
            }
            Object::Float(left % right)
        }
        op::LT => Object::Bool(left < right),
        op::LTEQ => Object::Bool(left <= right),
        op::GT => Object::Bool(left > right),
        op::GTEQ => Object::Bool(left >= right),
        
        _ => panic!(
            "Could not perform op {} on floats f{} and f{}",
            lookup(op).0,
            left,
            right,
        ),
    }
}

fn execute_bool_math(op: Op, left: bool, right: bool) -> Object {
    match op {
        op::BIT_AND => Object::Bool(left & right),
        op::BIT_OR => Object::Bool(left | right),
        op::BIT_XOR => Object::Bool(left ^ right),
        
        _ => panic!("Could not perform op {} on booleans", lookup(op).0),
    }
}

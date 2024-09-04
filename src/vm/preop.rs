use crate::{
    object::object::{Object, PreOps},
    op::{self, Op},
};

pub fn execute_pre_op(op: Op, operand: Object) -> Object {
    match op {
        op::NOT => operand.not(),
        op::NEGATE => operand.negate(),
        op::SIZE => operand.size(),
        op::HEAD => operand.head(),
        op::LAST => operand.last(),
        op::TAIL => operand.tail(),
        op::INIT => operand.init(),
        _ => unreachable!(),
    }
}

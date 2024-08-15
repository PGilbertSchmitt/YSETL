use crate::{
    object::{Object, ObjectOps},
    op::{self, Op},
};

pub fn execute_pre_op(op: Op, operand: Object) -> Object {
    match op {
        op::NOT => operand.not(),
        op::NEGATE => operand.negate(),

        op::SIZE => unimplemented!(),
        op::HEAD => unimplemented!(),
        op::LAST => unimplemented!(),
        op::TAIL => unimplemented!(),
        op::INIT => unimplemented!(),
        _ => unreachable!(),
    }
}

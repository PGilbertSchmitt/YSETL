use bytes::Bytes;

use crate::object::object::Object;

pub struct Frame {
    pub ins: Bytes,
    pub return_ptr: u64,
    pub stack_base: usize,
    pub closed_values: Vec<Object>,
}

impl Frame {
    pub fn new(ins: Bytes, return_ptr: u64, stack_base: usize, closed_values: Vec<Object>) -> Self {
        Self {
            ins,
            return_ptr,
            stack_base,
            closed_values,
        }
    }
}

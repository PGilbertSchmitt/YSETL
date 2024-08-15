use std::io::Cursor;

use bytes::{Buf, Bytes};

use crate::compiler::Bytecode;
use crate::object::{BaseObject, Object};
use crate::op;

pub mod frame;
pub mod binop;

const MAX_STACK_SIZE: usize = 4096;

pub struct VM {
    constants: Vec<Object>,
    instructions: Bytes,
    stack: Vec<Object>,
}

impl VM {
    pub fn new(bc: Bytecode) -> Self {
        VM {
            instructions: bc.instructions,
            constants: bc.constants.into_iter().map(BaseObject::wrap).collect(),
            stack: Vec::with_capacity(MAX_STACK_SIZE),
        }
    }

    /** Consume the VM to execute the entirety of the VM state */
    pub fn run(mut self) -> Option<Object> {
        let cur_ins = self.instructions;
        let mut ptr = Cursor::new(cur_ins);

        while ptr.has_remaining() {
            let op = ptr.get_u8();

            match op {
                op::CONST => {
                    let const_obj = self.constants[ptr.get_u16() as usize].clone();
                    self.stack.push(const_obj);
                }
                op::NULL => self.stack.push(BaseObject::Null.wrap()),
                op::TRUE => self.stack.push(BaseObject::True.wrap()),
                op::FALSE => self.stack.push(BaseObject::False.wrap()),
                op::ADD => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    match (left.as_ref(), right.as_ref()) {
                        (&BaseObject::Int(left), &BaseObject::Int(right)) => {
                            self.stack.push(BaseObject::Int(left + right).wrap());
                        }
                        _ => unimplemented!()
                    }
                }
                _ => unimplemented!()
            }
        }
        
        self.stack.pop()
    }

    // pub fn into_iter(self) -> VMIterator {
    //     VMIterator(self)
    // }
}

// pub struct VMIterator(VM);

// impl Iterator for VMIterator {
//     type Item = Option<Object>;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }
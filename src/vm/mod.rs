use std::io::Cursor;

use binop::execute_binop;
use bytes::{Buf, Bytes};
use preop::execute_pre_op;

use crate::compiler::bytecode::Bytecode;
use crate::object::{BaseObject, Object, ObjectOps};
use crate::op;

pub mod binop;
pub mod frame;
pub mod preop;

const MAX_STACK_SIZE: usize = 4096;

trait Stack {
    fn pop_one(&mut self) -> Object;
}

impl Stack for Vec<Object> {
    fn pop_one(&mut self) -> Object {
        self.pop()
            .unwrap_or_else(|| panic!("Called pop on an empty stack"))
    }
}

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
    pub fn run(mut self) {
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

                op::POP => {
                    self.stack.pop_one();
                }

                op::PRINT => {
                    println!("{}", self.stack.pop_one().to_s());
                }

                op::JUMP => {
                    let jmp_pos = ptr.get_u32();
                    ptr.set_position(jmp_pos as u64);
                }

                op::JUMP_NOT_TRUE => {
                    let jmp_pos = ptr.get_u32();
                    if !self.stack.pop_one().is_truthy() {
                        ptr.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_AND => {
                    let jmp_pos = ptr.get_u32();
                    if !self.stack.last().unwrap().is_truthy() {
                        ptr.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_OR => {
                    let jmp_pos = ptr.get_u32();
                    if self.stack.last().unwrap().is_truthy() {
                        ptr.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_NULL => {
                    let jmp_pos = ptr.get_u32();
                    if !self.stack.last().unwrap().is_null() {
                        ptr.set_position(jmp_pos as u64);
                    }
                }

                op::RETURN => todo!(),

                // Binary Operations (no jumps)
                op::TAKE
                | op::EXP
                | op::MULT
                | op::DIV
                | op::MOD
                | op::ADD
                | op::SUBTRACT
                | op::WITH_BIT_LEFT
                | op::LESS_BIT_RIGHT
                | op::BIT_AND
                | op::BIT_OR
                | op::BIT_XOR
                | op::IN
                | op::NOTIN
                | op::SUBSET
                | op::LT
                | op::LTEQ
                | op::GT
                | op::GTEQ
                | op::EQ
                | op::NEQ => {
                    let right = self.stack.pop_one();
                    let left = self.stack.pop_one();
                    self.stack.push(execute_binop(op, left, right))
                }

                // Prefix Operations
                op::NOT | op::NEGATE => {
                    let right = self.stack.pop_one();
                    self.stack.push(execute_pre_op(op, right))
                }

                op::DBG_PRINT_STACK_TOP => {
                    println!("Top of stack: {}", self.stack.last().unwrap().to_s())
                }

                _ => panic!("Still need to implement op {op}"),
            }
        }
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

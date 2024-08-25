use std::io::Cursor;

use super::binop::execute_binop;
use super::preop::execute_pre_op;
use bytes::{Buf, Bytes};

use crate::compiler::bytecode::Bytecode;
use crate::object::object::{BaseObject, Object, ObjectOps};
use crate::op;

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
    globals: Vec<Object>,

    null_ref: Object,
    true_ref: Object,
    false_ref: Object,
}

impl VM {
    pub fn new(bc: Bytecode) -> Self {
        let null_ref = BaseObject::Null.wrap();
        let true_ref = BaseObject::True.wrap();
        let false_ref = BaseObject::False.wrap();

        VM {
            instructions: bc.instructions,
            constants: bc.constants.into_iter().map(BaseObject::wrap).collect(),
            stack: Vec::with_capacity(MAX_STACK_SIZE),
            // Insertions can happen in any order, and uninitialized globals are hoisted, so
            // the globals vec is initialized to its known size with every space filled with null.
            globals: (0..bc.global_count)
                .into_iter()
                .map(|_| null_ref.clone())
                .collect(),

            null_ref,
            true_ref,
            false_ref,
        }
    }

    /** Consume the VM to execute the entirety of the VM state */
    pub fn run(mut self) {
        let cur_ins = self.instructions;
        let mut ins_cur = Cursor::new(cur_ins);

        while ins_cur.has_remaining() {
            let op = ins_cur.get_u8();

            match op {
                op::CONST => {
                    let const_obj = self.constants[ins_cur.get_u16() as usize].clone();
                    self.stack.push(const_obj);
                }
                op::NULL => self.stack.push(self.null_ref.clone()),
                op::TRUE => self.stack.push(self.true_ref.clone()),
                op::FALSE => self.stack.push(self.false_ref.clone()),

                op::SET_GLOBAL => {
                    let global_ptr = ins_cur.get_u16();
                    self.globals[global_ptr as usize] = self.stack.pop_one();
                }

                op::GET_GLOBAL => {
                    let global_ptr = ins_cur.get_u16();
                    self.stack.push(self.globals[global_ptr as usize].clone());
                }

                op::SET_LOCAL => todo!(),
                op::GET_LOCAL => todo!(),

                op::MAKE_LIT_COL => {
                    let _flag = ins_cur.get_u8();
                    let _size = ins_cur.get_u16();
                    todo!();
                }

                op::MAKE_RN_COL => todo!(),

                op::POP => {
                    self.stack.pop_one();
                }

                op::JUMP => {
                    let jmp_pos = ins_cur.get_u32();
                    ins_cur.set_position(jmp_pos as u64);
                }

                op::JUMP_NOT_TRUE => {
                    let jmp_pos = ins_cur.get_u32();
                    if !self.stack.pop_one().is_truthy() {
                        ins_cur.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_AND => {
                    let jmp_pos = ins_cur.get_u32();
                    if !self.stack.last().unwrap().is_truthy() {
                        ins_cur.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_OR => {
                    let jmp_pos = ins_cur.get_u32();
                    if self.stack.last().unwrap().is_truthy() {
                        ins_cur.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_NULL => {
                    let jmp_pos = ins_cur.get_u32();
                    if !self.stack.last().unwrap().is_null() {
                        ins_cur.set_position(jmp_pos as u64);
                    }
                }

                op::RETURN => todo!(),

                op::PRINT => {
                    println!("{}", self.stack.pop_one().to_s());
                }

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

use std::collections::HashSet;
use std::io::Cursor;

use super::binop::execute_binop;
use super::frame::Frame;
use super::preop::execute_pre_op;
use bytes::Buf;

use crate::compiler::bytecode::{Bytecode, TUP_BASE};
use crate::object::object::{BaseObject, Executor, Object, ObjectOps};
use crate::op;

const MAX_STACK_SIZE: usize = 4096;

trait Stack {
    fn pop_one(&mut self) -> Object;
}

impl Stack for Vec<Object> {
    fn pop_one(&mut self) -> Object {
        self.pop().expect("Called pop on an empty stack")
    }
}

pub struct VM {
    constants: Vec<Object>,
    base_iterators: Vec<Executor>,
    frames: Vec<Frame>,
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
            frames: vec![Frame::new_as_func(bc.instructions, 0, 0, vec![])],
            constants: bc.constants.into_iter().map(BaseObject::wrap).collect(),
            base_iterators: bc.iterators,
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
        let mut cursor = Cursor::new(self.frame().ins.clone());

        while cursor.has_remaining() {
            let op = cursor.get_u8();

            match op {
                op::CONST => {
                    let const_obj = self.constants[cursor.get_u16() as usize].clone();
                    self.stack.push(const_obj);
                }
                op::NULL => self.stack.push(self.null_ref.clone()),
                op::TRUE => self.stack.push(self.true_ref.clone()),
                op::FALSE => self.stack.push(self.false_ref.clone()),

                op::SET_GLOBAL => {
                    let global_ptr = cursor.get_u16();
                    self.globals[global_ptr as usize] = self.stack.pop_one();
                }

                op::GET_GLOBAL => {
                    let global_ptr = cursor.get_u16();
                    self.stack.push(self.globals[global_ptr as usize].clone());
                }

                op::SET_LOCAL => {
                    let stack_offset = cursor.get_u16() as usize;
                    let stack_location = self.frame().stack_base + stack_offset;
                    self.stack[stack_location] = self.stack.pop().unwrap();
                }

                op::GET_LOCAL => {
                    let stack_offset = cursor.get_u16() as usize;
                    let stack_location = self.frame().stack_base + stack_offset;
                    self.stack.push(self.stack[stack_location].clone());
                }

                op::GET_LOCKED => {
                    let closed_value_idx = cursor.get_u16() as usize;
                    self.stack.push(
                        self.frame()
                            .closed_values
                            .get(closed_value_idx)
                            .unwrap()
                            .clone(),
                    );
                }

                op::MAKE_LIT_COL => {
                    let flag = cursor.get_u8();
                    let size = cursor.get_u16() as usize;
                    let stack_start_ptr = self.stack.len() - size;
                    let elements: Vec<Object> = self.stack.drain(stack_start_ptr..).collect();
                    if flag & TUP_BASE == 0 {
                        self.stack.push(BaseObject::Set(HashSet::from_iter(elements)).wrap())
                    } else {
                        self.stack.push(BaseObject::Tuple(elements).wrap());
                    }
                }

                op::MAKE_RN_COL => {
                    let flag = cursor.get_u8();
                    let range_end = self.stack.pop_one().inner_int();
                    let range_start = self.stack.pop_one().inner_int();
                    // TODO: This is incomplete. There are several interactions with inclusive/exclusive and
                    // low->high/high->low ranges.
                    let elements: Vec<Object> = (range_start..range_end)
                        .map(|i| BaseObject::Int(i).wrap())
                        .collect();
                    if flag & TUP_BASE != 0 {
                        self.stack.push(BaseObject::Tuple(elements).wrap());
                    } else {
                        todo!();
                    }
                }

                op::MAKE_FN => {
                    let const_ptr = cursor.get_u16() as usize;
                    let locked_param_count = cursor.get_u16() as usize;
                    let function = self.constants[const_ptr as usize].clone();
                    let params_start = self.stack.len() - locked_param_count;
                    let (mut function, num_req_params, num_opt_params) = function.inner_fn();
                    function.locked_values = self.stack.drain(params_start..).collect();
                    self.stack.push(
                        BaseObject::Closure {
                            function: Box::new(function),
                            num_req_params,
                            num_opt_params,
                        }
                        .wrap(),
                    );
                }

                op::POP => {
                    self.stack.pop_one();
                }

                op::JUMP => {
                    let jmp_pos = cursor.get_u32();
                    cursor.set_position(jmp_pos as u64);
                }

                op::JUMP_IF_FALSE => {
                    let jmp_pos = cursor.get_u32();
                    if !self.stack.pop_one().is_truthy() {
                        cursor.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_AND => {
                    let jmp_pos = cursor.get_u32();
                    if !self.stack.last().unwrap().is_truthy() {
                        cursor.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_OR => {
                    let jmp_pos = cursor.get_u32();
                    if self.stack.last().unwrap().is_truthy() {
                        cursor.set_position(jmp_pos as u64);
                    }
                }

                op::JUMP_PEEK_NULL => {
                    let jmp_pos = cursor.get_u32();
                    if !self.stack.last().unwrap().is_null() {
                        cursor.set_position(jmp_pos as u64);
                    }
                }

                op::CALL => {
                    let arg_count = cursor.get_u16() as usize;
                    let (fn_obj, num_req_params, num_opt_params) = self.stack
                        [self.stack.len() - arg_count - 1]
                        .clone()
                        .inner_fn();
                    let total_params = num_req_params + num_opt_params;
                    if arg_count < num_req_params {
                        panic!("Didn't provide enough arguments to function");
                    } else if arg_count > total_params {
                        panic!("Provided too many arguments to function");
                    }

                    let base_pointer = self.stack.len() - arg_count;

                    // The optional params must be on the stack whether or not the args are passed,
                    // initialized to null. We also need null-initialied spaces all non-param
                    // local variables.
                    let unaccounted_count = fn_obj.num_locals + total_params - arg_count;
                    for _ in 0..unaccounted_count {
                        self.stack.push(self.null_ref.clone());
                    }

                    self.frames.push(Frame::new_as_func(
                        fn_obj.ins.clone(),
                        cursor.position(),
                        base_pointer,
                        fn_obj.locked_values.clone(),
                    ));
                    cursor = Cursor::new(fn_obj.ins);
                }

                op::RETURN => {
                    let last_frame = self.frames.pop().unwrap();
                    cursor = Cursor::new(self.frame().ins.clone());
                    cursor.set_position(last_frame.return_ptr);
                    let return_value = self.stack.pop().unwrap();
                    self.stack.truncate(last_frame.stack_base); // Remove all args and local vars
                    self.stack.pop(); // Remove the called function
                    self.stack.push(return_value);
                }

                op::ITER_START => {
                    let iter_idx = cursor.get_u16();
                    let locked_param_count = cursor.get_u16() as usize;
                    let type_flag = cursor.get_u8();
                    let params_start = self.stack.len() - locked_param_count;
                    let iterator = self.base_iterators.get(iter_idx as usize).unwrap();
                    let closed_values = self.stack.drain(params_start..).collect();
                    let base_pointer = self.stack.len();

                    // Space for the locals to exist on the stack
                    for _ in 0..iterator.num_locals {
                        self.stack.push(self.false_ref.clone());
                    }

                    self.frames.push(Frame::new_as_iter(
                        iterator.ins.clone(),
                        cursor.position(),
                        base_pointer,
                        closed_values,
                        type_flag == TUP_BASE,
                    ));
                    cursor = Cursor::new(iterator.ins.clone());
                }

                op::ITER_NEXT => {
                    let iter_idx = cursor.get_u8() as usize;
                    let jmp_ptr = cursor.get_u32() as u64;
                    self.frame_mut()
                        .iter_next(iter_idx, || cursor.set_position(jmp_ptr));
                }

                op::ITER_COLLECT => {
                    let item = self.stack.pop_one();
                    self.frame_mut().iter_collect(item);
                }

                op::ITER_END => {
                    let last_frame = self.frames.pop().unwrap();
                    cursor = Cursor::new(self.frame().ins.clone());
                    cursor.set_position(last_frame.return_ptr);
                    self.stack.truncate(last_frame.stack_base); // Remove local vars
                    self.stack.push(last_frame.collector());
                }

                op::MAKE_ITER => {
                    let collection = self.stack.pop_one();
                    self.frame_mut().make_iter(&collection);
                }

                op::DUP_ITER => {
                    self.frame_mut().dup_iter();
                }

                op::GET_ITER_VAL => {
                    let iter_idx = cursor.get_u8() as usize;
                    self.stack.push(self.frame().get_iter_val(iter_idx));
                }

                op::GET_ITER_KEY => {
                    let iter_idx = cursor.get_u8() as usize;
                    self.stack.push(self.frame().get_iter_key(iter_idx));
                }

                op::ITER_EMPTY_CHECK => {
                    let jump_pos = cursor.get_u32();
                    if self.frame().any_iter_empty() {
                        cursor.set_position(jump_pos as u64);
                    }
                }

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

    fn frame(&self) -> &Frame {
        self.frames.last().expect("There are no frames!")
    }

    fn frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("There are no frames!")
    }
}

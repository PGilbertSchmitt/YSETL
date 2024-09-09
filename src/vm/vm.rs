use std::rc::Rc;

use super::binop::execute_binop;
use super::frame::{Collector, Frame};
use super::preop::execute_pre_op;
use bytes::Bytes;

use crate::compiler::bytecode::flags::RED_OP_BIT;
use crate::compiler::bytecode::{
    flags::{SET_BASE, STEP_BIT, TUP_BASE},
    Bytecode,
};
use crate::object::object::{Atom, Executor, Object, ObjectOps};
use crate::op::{self, lookup};

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
    atoms: Vec<Atom>,

    null_ref: Object,
    true_ref: Object,
    false_ref: Object,
}

impl VM {
    pub fn new(bc: Bytecode) -> Self {
        let null_ref = Object::Null;
        let true_ref = Object::Bool(true);
        let false_ref = Object::Bool(false);

        VM {
            frames: vec![Frame::new_as_func(bc.instructions, 0, 0, Rc::new(vec![]))],
            constants: bc.constants,
            base_iterators: bc.iterators,
            stack: Vec::with_capacity(MAX_STACK_SIZE),
            // Insertions can happen in any order, and uninitialized globals are hoisted, so
            // the globals vec is initialized to its known size with every space filled with null.
            globals: (0..bc.global_count)
                .into_iter()
                .map(|_| null_ref.clone())
                .collect(),
            atoms: bc.atoms,

            null_ref,
            true_ref,
            false_ref,
        }
    }

    /** Consume the VM to execute the entirety of the VM state */
    pub fn run(mut self) {
        let mut i_ptr: usize = 0;
        let mut ins = self.frame().ins.clone();

        while i_ptr < ins.len() {
            let op = ins[i_ptr];
            i_ptr += 1;

            match op {
                op::CONST => {
                    let const_ptr = Self::read_u16(&ins, i_ptr) as usize;
                    let const_obj = self.constants[const_ptr].clone();
                    i_ptr += 2;
                    self.stack.push(const_obj);
                }
                op::NULL => self.stack.push(self.null_ref.clone()),
                op::TRUE => self.stack.push(self.true_ref.clone()),
                op::FALSE => self.stack.push(self.false_ref.clone()),

                op::SET_GLOBAL => {
                    let global_ptr = Self::read_u16(&ins, i_ptr);
                    i_ptr += 2;
                    self.globals[global_ptr as usize] = self.stack.pop_one();
                }

                op::GET_GLOBAL => {
                    let global_ptr = Self::read_u16(&ins, i_ptr);
                    i_ptr += 2;
                    self.stack.push(self.globals[global_ptr as usize].clone());
                }

                op::SET_LOCAL => {
                    let stack_offset = Self::read_u16(&ins, i_ptr) as usize;
                    i_ptr += 2;
                    let stack_location = self.frame().stack_base + stack_offset;
                    self.stack[stack_location] = self.stack.pop().unwrap();
                }

                op::GET_LOCAL => {
                    let stack_offset = Self::read_u16(&ins, i_ptr) as usize;
                    i_ptr += 2;
                    let stack_location = self.frame().stack_base + stack_offset;
                    self.stack.push(self.stack[stack_location].clone());
                }

                op::GET_LOCKED => {
                    let closed_value_idx = Self::read_u16(&ins, i_ptr) as usize;
                    i_ptr += 2;
                    self.stack
                        .push(self.frame().closed_values[closed_value_idx].clone());
                }

                op::MAKE_LIT_COL => {
                    let flag = ins[i_ptr];
                    let size = Self::read_u16(&ins, i_ptr + 1) as usize;
                    i_ptr += 3;
                    let stack_start_ptr = self.stack.len() - size;
                    let elements: Vec<Object> = self.stack.drain(stack_start_ptr..).collect();

                    self.stack.push(if flag & TUP_BASE == 0 {
                        Object::new_set_from_vec(elements)
                    } else {
                        Object::new_tuple(elements)
                    })
                }

                op::MAKE_RN_COL => {
                    let flag = ins[i_ptr];
                    i_ptr += 1;
                    let range_end = self.stack.pop_one().inner_int();
                    let range_start = self.stack.pop_one().inner_int();

                    let step = if flag & STEP_BIT != 0 {
                        let step = self.stack.pop_one().inner_int();
                        Some(step as usize)
                    } else {
                        None
                    };
                    self.stack
                        .push(Object::make_range(range_start, range_end, step, flag))
                }

                op::GET_ATOM => {
                    let atom_idx = Self::read_u32(&ins, i_ptr) as usize;
                    i_ptr += 4;
                    self.stack.push(Object::Atom(self.atoms[atom_idx].clone()));
                }

                op::MAKE_ATOM => {
                    let atom_name = Atom::gen_atom_name();
                    let atom = Atom::new(self.atoms.len() as u32, atom_name);
                    self.atoms.push(atom.clone());
                    self.stack.push(Object::Atom(atom));
                }

                op::MAKE_FN => {
                    let const_ptr = Self::read_u16(&ins, i_ptr) as usize;
                    let locked_param_count = Self::read_u16(&ins, i_ptr + 2) as usize;
                    i_ptr += 4;
                    let function = self.constants[const_ptr as usize].clone();
                    let params_start = self.stack.len() - locked_param_count;
                    let (mut function, num_req_params, num_opt_params) = function.inner_fn();
                    function.locked_values = Rc::new(self.stack.drain(params_start..).collect());
                    self.stack.push(Object::new_closure(
                        function,
                        num_req_params,
                        num_opt_params,
                    ));
                }

                op::POP => {
                    self.stack.pop_one();
                }

                op::JUMP => {
                    let jmp_pos = Self::read_u32(&ins, i_ptr);
                    i_ptr = jmp_pos as usize;
                }

                op::JUMP_IF_FALSE => {
                    let jmp_pos = Self::read_u32(&ins, i_ptr);
                    if !self.stack.pop_one().is_truthy() {
                        i_ptr = jmp_pos as usize;
                    } else {
                        i_ptr += 4;
                    }
                }

                op::JUMP_IF_TRUE => {
                    let jmp_pos = Self::read_u32(&ins, i_ptr);
                    if self.stack.pop_one().is_truthy() {
                        i_ptr = jmp_pos as usize;
                    } else {
                        i_ptr += 4;
                    }
                }

                op::JUMP_PEEK_AND => {
                    let jmp_pos = Self::read_u32(&ins, i_ptr);
                    if !self.stack.last().unwrap().is_truthy() {
                        i_ptr = jmp_pos as usize;
                    } else {
                        i_ptr += 4;
                    }
                }

                op::JUMP_PEEK_OR => {
                    let jmp_pos = Self::read_u32(&ins, i_ptr);
                    if self.stack.last().unwrap().is_truthy() {
                        i_ptr = jmp_pos as usize;
                    } else {
                        i_ptr += 4;
                    }
                }

                op::JUMP_PEEK_NULL => {
                    let jmp_pos = Self::read_u32(&ins, i_ptr);
                    if !self.stack.last().unwrap().is_null() {
                        i_ptr = jmp_pos as usize;
                    } else {
                        i_ptr += 4;
                    }
                }

                op::CALL => {
                    let arg_count = Self::read_u16(&ins, i_ptr) as usize;
                    i_ptr += 2;
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
                        i_ptr,
                        base_pointer,
                        fn_obj.locked_values.clone(),
                    ));
                    ins = fn_obj.ins;
                    i_ptr = 0;
                }

                op::RETURN => {
                    let last_frame = self.frames.pop().unwrap();
                    ins = self.frame().ins.clone();
                    i_ptr = last_frame.return_ptr;
                    let return_value = self.stack.pop().unwrap();
                    self.stack.truncate(last_frame.stack_base); // Remove all args and local vars
                    self.stack.pop(); // Remove the called function
                    self.stack.push(return_value);
                }

                op::ITER_START => {
                    let iter_idx = Self::read_u16(&ins, i_ptr);
                    let locked_param_count = Self::read_u16(&ins, i_ptr + 2) as usize;
                    let collection_count = ins[i_ptr + 4] as usize;
                    let type_flag = ins[i_ptr + 5];
                    i_ptr += 6;

                    let params_start = self.stack.len() - locked_param_count;
                    let iterator = &self.base_iterators[iter_idx as usize];
                    let closed_values = Rc::new(self.stack.drain(params_start..).collect());
                    let collections_start = params_start - collection_count;
                    let collections = self.stack.drain(collections_start..).collect();

                    let mut reducer: Option<Object> = None;
                    let collector = if type_flag & TUP_BASE != 0 {
                        Collector::new_tuple()
                    } else if type_flag & SET_BASE != 0 {
                        Collector::new_set()
                    } else {
                        // In this situation, the initial accumulator will be on the top of the stack. If the reducer
                        // is an expression reducer, there will also be a reducer function under the initial accumulator.
                        // The op-based reducer will only have the accumulator.
                        let init = self.stack.pop_one();
                        if type_flag & RED_OP_BIT == 0 {
                            let reducer_fn = self.stack.pop_one();
                            if !reducer_fn.can_reduce() {
                                panic!("Provided reducer has incorrect param counts");
                            }
                            reducer = Some(reducer_fn);
                        }
                        Collector::Accum(init)
                    };

                    let base_pointer = self.stack.len();
                    // Space for the locals to exist on the stack
                    for _ in 0..iterator.num_locals {
                        self.stack.push(self.null_ref.clone());
                    }

                    self.frames.push(Frame::new_as_iter(
                        iterator.ins.clone(),
                        i_ptr,
                        base_pointer,
                        closed_values,
                        &collections,
                        collector,
                        reducer,
                    ));
                    ins = iterator.ins.clone();
                    i_ptr = 0;
                }

                op::GET_ACC => {
                    self.stack.push(self.frame().get_collection());
                }

                op::REDUCE_CALL => {
                    let reducer = self.frame().get_reducer();
                    let (executor, ..) = reducer.inner_fn();
                    let stack_base = self.stack.len() - 2;
                    self.frames.push(Frame::new_as_func(
                        executor.ins.clone(),
                        i_ptr,
                        stack_base,
                        // This could be an option, but it's already kinda unweildy
                        Rc::new(Vec::new()),
                    ));
                    ins = executor.ins;
                    i_ptr = 0;
                }

                op::REDUCE_WITH => {
                    let op = ins[i_ptr];
                    i_ptr += 1;
                    let right = self.stack.pop_one();
                    let left = self.stack.pop_one();
                    self.frame_mut()
                        .iter_collect(execute_binop(op, &left, &right).unwrap());
                }

                op::ITER_NEXT => {
                    let iter_idx = ins[i_ptr] as usize;
                    let jmp_ptr = Self::read_u32(&ins, i_ptr + 1) as usize;
                    // This might be bad
                    if self.frame_mut().iter_next(iter_idx) {
                        i_ptr = jmp_ptr;
                    } else {
                        i_ptr += 5;
                    };
                }

                op::ITER_COLLECT => {
                    let item = self.stack.pop_one();
                    self.frame_mut().iter_collect(item);
                }

                op::ITER_END => {
                    let last_frame = self.frames.pop().unwrap();
                    ins = self.frame().ins.clone();
                    i_ptr = last_frame.return_ptr;
                    self.stack.truncate(last_frame.stack_base); // Remove local vars
                    self.stack.push(last_frame.into_collector());
                }

                op::DUP_ITER => {
                    self.stack.push(self.stack.last().unwrap().clone());
                }

                op::GET_ITER_VAL => {
                    let iter_idx = ins[i_ptr] as usize;
                    i_ptr += 1;
                    self.stack.push(self.frame().get_iter_val(iter_idx));
                }

                op::GET_ITER_KEY => {
                    let iter_idx = ins[i_ptr] as usize;
                    i_ptr += 1;
                    self.stack.push(self.frame().get_iter_key(iter_idx));
                }

                op::ITER_EMPTY_CHECK => {
                    let jump_pos = Self::read_u32(&ins, i_ptr);
                    i_ptr += 4;
                    if self.frame().any_iter_empty() {
                        i_ptr = jump_pos as usize;
                    };
                }

                op::PRINT => {
                    println!("{}", self.stack.pop_one().to_s());
                }

                op::EQ => {
                    let right = self.stack.pop_one();
                    let left = self.stack.pop_one();
                    self.stack.push(Object::Bool(left == right));
                }

                op::NEQ => {
                    let right = self.stack.pop_one();
                    let left = self.stack.pop_one();
                    self.stack.push(Object::Bool(left != right));
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
                | op::GTEQ => {
                    let right = self.stack.pop_one();
                    let left = self.stack.pop_one();
                    self.stack.push(execute_binop(op, &left, &right).unwrap());
                }

                // Prefix Operations
                op::NOT | op::NEGATE | op::SIZE | op::HEAD | op::LAST | op::TAIL | op::INIT => {
                    let right = self.stack.pop_one();
                    self.stack.push(execute_pre_op(op, right));
                }

                op::DBG_PRINT_STACK_TOP => {
                    println!("Top of stack: {}", self.stack.last().unwrap().to_s());
                }

                _ => {
                    println!("Still need to implement op {op}. Attempting lookup:");
                    let (name, widths) = lookup(op);
                    println!("Lookup value: {name} [{widths:?}]");
                }
            }
        }
    }

    fn frame(&self) -> &Frame {
        self.frames.last().expect("There are no frames!")
    }

    fn frame_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("There are no frames!")
    }

    fn read_u16(ins: &Bytes, ptr: usize) -> u16 {
        ((ins[ptr] as u16) << 8) ^ ins[ptr + 1] as u16
    }

    fn read_u32(ins: &Bytes, ptr: usize) -> u32 {
        ((ins[ptr] as u32) << 24)
            ^ ((ins[ptr + 1] as u32) << 16)
            ^ ((ins[ptr + 2] as u32) << 8)
            ^ ins[ptr + 3] as u32
    }
}

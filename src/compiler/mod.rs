use crate::{
    object::BaseObject,
    parser::ast::{BinOp, Expr, Stmt},
};

use super::op::{self, Op};
use bytes::{BufMut, Bytes, BytesMut};

pub struct Compiler {
    instructions: BytesMut,
    constants: Vec<BaseObject>,
}

pub struct Bytecode {
    pub instructions: Bytes,
    pub constants: Vec<BaseObject>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: BytesMut::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile_expr(&mut self, node: Expr) {
        match node {
            Expr::Null => self.emit(op::NULL),
            Expr::True => self.emit(op::TRUE),
            Expr::False => self.emit(op::FALSE),
            Expr::Integer(value) => {
                let const_ptr = self.add_const(BaseObject::Int(value));
                self.emit_with_u16(op::CONST, const_ptr);
            }
            Expr::Float(value) => {
                let const_ptr = self.add_const(BaseObject::Float(value));
                self.emit_with_u16(op::CONST, const_ptr);
            }
            Expr::String(value) => {
                let const_ptr = self.add_const(BaseObject::String(value));
                self.emit_with_u16(op::CONST, const_ptr);
            }
            Expr::Infix { op, lhs, rhs } => {
                self.compile_expr(*rhs);
                self.compile_expr(*lhs);
                self.emit(binop_to_op(op));
            }
            _ => unimplemented!(),
        };
    }

    pub fn compile_stmt(&mut self, node: Stmt) {
        match node {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.emit(op::POP);
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.emit(op::PRINT);
            }
            _ => unimplemented!(),
        }
    }

    pub fn finish(self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.freeze(),
            constants: self.constants,
        }
    }

    fn emit(&mut self, code: Op) {
        self.instructions.put_u8(code);
    }

    // fn emit_bytes(&mut self, bytes: Bytes) {
    //     self.instructions.put(bytes);
    // }

    fn emit_with_u16(&mut self, code: Op, operand: u16) {
        self.emit(code);
        self.instructions.put_u16(operand);
    }

    fn add_const(&mut self, base_object: BaseObject) -> u16 {
        self.constants.push(base_object);
        (self.constants.len() - 1)
            .try_into()
            .unwrap_or_else(|_| panic!("Too many constants were generated"))
    }

    
}

fn binop_to_op(binop: BinOp) -> Op {
    match binop {
        BinOp::Add => op::ADD,
        _ => unimplemented!(),
    }
}

use crate::{
    object::BaseObject,
    parser::ast::{BinOp, Expr, Stmt, StmtList},
};
use super::op::{self, Op};
use bytecode::Bytecode;
use bytes::{BufMut, BytesMut};

pub mod bytecode;

pub struct Compiler {
    instructions: BytesMut,
    constants: Vec<BaseObject>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: BytesMut::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile_program(mut self, stmts: StmtList) -> Bytecode {
        self.compile_stmt_list(stmts);
        self.finish()
    }

    pub fn finish(self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.freeze(),
            constants: self.constants,
        }
    }

    fn compile_stmt_list(&mut self, stmts: StmtList) {
        stmts.into_iter().for_each(|stmt| self.compile_stmt(stmt));
    }

    fn compile_stmt(&mut self, node: Stmt) {
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

    fn compile_expr(&mut self, node: Expr) {
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
                self.compile_expr(*lhs);
                self.compile_expr(*rhs);
                self.emit(binop_to_op(op));
            }
            _ => unimplemented!(),
        };
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
        BinOp::Nullcoel => op::NULLCOEL,
        BinOp::Take => op::TAKE,
        BinOp::Exp => op::EXP,
        BinOp::Mult => op::MULT,
        BinOp::Div => op::DIV,
        BinOp::Mod => op::MOD,
        BinOp::Add => op::ADD,
        BinOp::Subtract => op::SUBTRACT,
        BinOp::WithBitLeft => op::WITH_BIT_LEFT,
        BinOp::LessBitRight => op::LESS_BIT_RIGHT,
        BinOp::BitAnd => op::BIT_AND,
        BinOp::BitOr => op::BIT_OR,
        BinOp::BitXor => op::BIT_XOR,
        BinOp::In => op::IN,
        BinOp::Notin => op::NOTIN,
        BinOp::Subset => op::SUBSET,
        BinOp::Lt | BinOp::Gt => op::LT,
        BinOp::Lteq | BinOp::Gteq => op::LTEQ,
        BinOp::Eq => op::EQ,
        BinOp::Neq => op::NEQ,
        BinOp::And => op::LOGICAL_AND,
        BinOp::Or => op::LOGICAL_OR,
        BinOp::Impl => op::LOGICAL_IMPL,
    }
}

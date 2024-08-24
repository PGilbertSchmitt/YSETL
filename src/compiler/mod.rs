use super::op::{self, Op};
use crate::{
    object::BaseObject, op::MAKE_RN_COL, parser::ast::{BinOp, Bound, Expr, Former, PreOp, Stmt, StmtList}
};
use bytecode::{Bytecode, INCL_BIT, SET_BASE, STEP_BIT, TUP_BASE};
use bytes::{BufMut, Bytes, BytesMut};
use symbols::{Scope, SymbolStack};

pub mod bytecode;
pub mod symbols;

pub struct Compiler {
    instructions: BytesMut,
    constants: Vec<BaseObject>,
    symbols: SymbolStack,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: BytesMut::new(),
            constants: Vec::new(),
            symbols: SymbolStack::new(),
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
            global_count: self.symbols.size(),
        }
    }

    fn compile_stmt_list(&mut self, stmts: StmtList) {
        stmts.into_iter().for_each(|stmt| self.compile_stmt(stmt));
    }

    // Should keep the stack clean after execution
    fn compile_stmt(&mut self, node: Stmt) {
        match node {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                self.emit(op::POP);
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.emit(op::PRINT); // Performs a pop
            }
            Stmt::Assign { target, value } => {
                match target {
                    Bound::Ident(ident) => {
                        self.compile_expr(value);
                        let sym = self.symbols.register_sym(ident);
                        let code = match sym.scope {
                            Scope::GLOBAL => op::SET_GLOBAL,
                            Scope::LOCAL => op::SET_LOCAL,
                        };
                        self.emit_with_u16(code, sym.index);
                    }
                    // Gonna need to have a hard thing about destructuring via stack
                    _ => unimplemented!(),
                }
            }
            _ => panic!("Unimplemented in compile_stmt: {node:?}"),
        }
    }

    // Should leave something on the stack, which may be used by conditional expressions
    // Non-expression statements should leave a `null` on the stack
    fn compile_stmt_like_expr(&mut self, node: Stmt) {
        match node {
            Stmt::Expr(expr) => self.compile_expr(expr),
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.emit(op::PRINT);
                self.emit(op::NULL);
            }
            _ => panic!("Unimplemented in compile_stmt_like_expr: {node:?}"),
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
            Expr::Tuple(former) => self.compile_former(former, TUP_BASE),
            Expr::Set(former) => self.compile_former(former, SET_BASE),
            Expr::Ident(name) => {
                let sym = self.symbols.lookup(&name).unwrap_or_else(||
                    panic!("Symbol \"{name}\" was never declared")
                );
                let code = match sym.scope {
                    Scope::GLOBAL => op::GET_GLOBAL,
                    Scope::LOCAL => op::GET_LOCAL,
                };
                self.emit_with_u16(code, sym.index);
            }
            Expr::Infix { op, lhs, rhs } => match op {
                BinOp::And => self.compile_binary_op_with_jump(op::JUMP_PEEK_AND, lhs, rhs),
                BinOp::Or => self.compile_binary_op_with_jump(op::JUMP_PEEK_OR, lhs, rhs),
                BinOp::Nullcoel => self.compile_binary_op_with_jump(op::JUMP_PEEK_NULL, lhs, rhs),
                _ => {
                    self.compile_expr(*lhs);
                    self.compile_expr(*rhs);
                    self.emit(from_binop(op));
                }
            },
            Expr::Prefix { op, rhs } => {
                self.compile_expr(*rhs);
                from_pre_op(op).map(|code| self.emit(code));
            }
            Expr::Ternary {
                condition,
                consequence,
                alternative,
            } => {
                self.compile_expr(*condition);

                let jmp_nt_operand_ptr = self.ins_len() + 1;
                self.emit_with_u32(op::JUMP_NOT_TRUE, u32::MAX);
                self.compile_stmt_like_expr(*consequence);

                let jmp_operand_ptr = self.ins_len() + 1;
                self.emit_with_u32(op::JUMP, u32::MAX);
                let jnt_destination = self.ins_len();
                self.compile_stmt_like_expr(*alternative);
                let jmp_destination = self.ins_len();

                // Set correct jump locations
                self.overwrite_u32(jmp_nt_operand_ptr, jnt_destination as u32);
                self.overwrite_u32(jmp_operand_ptr, jmp_destination as u32)
            }
            _ => panic!("Unimplemented in compile_expr: {node:?}"),
        };
    }

    fn compile_binary_op_with_jump(&mut self, op: Op, lhs: Box<Expr>, rhs: Box<Expr>) {
        self.compile_expr(*lhs);
        let jmp_operand_ptr = self.ins_len() + 1;
        self.emit_with_u32(op, u32::MAX);
        self.compile_expr(*rhs);
        let jmp_destination = self.ins_len();
        self.overwrite_u32(jmp_operand_ptr, jmp_destination as u32);
    }

    fn compile_former(&mut self, former: Former, flag_base: u8) {
        match former {
            Former::Empty => self.emit_with_u8_u16(op::MAKE_LIT_COL, flag_base, 0),
            Former::Literal(elements) => {
                let size = elements.len() as u16;
                elements.into_iter().for_each(|el| self.compile_expr(el));
                self.emit_with_u8_u16(op::MAKE_LIT_COL, flag_base, size);
            },
            Former::Range(range) => {
                let mut flag = flag_base;
                if range.inclusive { flag |= INCL_BIT };
                if let Some(step) = range.step {
                    self.compile_expr(*step);
                    flag |= STEP_BIT;
                };

                self.compile_expr(*range.start);
                self.compile_expr(*range.end);
                self.emit_with_u8(MAKE_RN_COL, flag);
            }
            Former::Iterator { output: _, iterator: _ } => unimplemented!(),
        }
    }

    /* Buncha convenience methods for modifying the instructions */

    fn emit(&mut self, code: Op) {
        self.instructions.put_u8(code);
    }

    fn emit_with_u8(&mut self, code: Op, operand: u8) {
        self.emit(code);
        self.instructions.put_u8(operand);
    }

    fn emit_with_u8_u16(&mut self, code: Op, operand_1: u8, operand_2: u16) {
        self.emit(code);
        self.instructions.put_u8(operand_1);
        self.instructions.put_u16(operand_2);
    }

    fn emit_with_u16(&mut self, code: Op, operand: u16) {
        self.emit(code);
        self.instructions.put_u16(operand);
    }

    fn emit_with_u32(&mut self, code: Op, operand: u32) {
        self.emit(code);
        self.instructions.put_u32(operand);
    }

    // fn emit_bytes(&mut self, bytes: Bytes) {
    //     self.instructions.put(bytes);
    // }

    fn add_const(&mut self, base_object: BaseObject) -> u16 {
        self.constants.push(base_object);
        (self.constants.len() - 1)
            .try_into()
            .unwrap_or_else(|_| panic!("Too many constants were generated"))
    }

    fn ins_len(&self) -> usize {
        self.instructions.len()
    }

    fn overwrite(&mut self, at: usize, data: Bytes) {
        for (i, byte) in data.into_iter().enumerate() {
            self.instructions[at + i] = byte;
        }
    }

    fn overwrite_u32(&mut self, at: usize, value: u32) {
        let mut data = BytesMut::with_capacity(4);
        data.put_u32(value);
        self.overwrite(at, data.freeze());
    }
}

fn from_binop(binop: BinOp) -> Op {
    match binop {
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
        BinOp::Lt => op::LT,
        BinOp::Gt => op::GT,
        BinOp::Lteq => op::LTEQ,
        BinOp::Gteq => op::GTEQ,
        BinOp::Eq => op::EQ,
        BinOp::Neq => op::NEQ,
        BinOp::Impl => op::LOGICAL_IMPL,
        _ => unreachable!(),
    }
}

fn from_pre_op(pre_op: PreOp) -> Option<Op> {
    match pre_op {
        PreOp::Not => Some(op::NOT),
        PreOp::Negate => Some(op::NEGATE),
        PreOp::Size => Some(op::SIZE),
        PreOp::Head => Some(op::HEAD),
        PreOp::Last => Some(op::LAST),
        PreOp::Tail => Some(op::TAIL),
        PreOp::Init => Some(op::INIT),
        PreOp::Identity => None, // No op needed for Identity
    }
}

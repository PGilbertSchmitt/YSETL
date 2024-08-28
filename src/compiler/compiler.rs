use super::bytecode::{Bytecode, INCL_BIT, SET_BASE, STEP_BIT, TUP_BASE};
use super::scope::{ScopeKind, ScopeStack, SymbolRef};

use crate::object::object::{BaseObject, Function};
use crate::op::{self, Op};
use crate::parser::ast::{BinOp, Bound, Expr, ExprList, Former, Postfix, PreOp, Stmt, StmtList};
use bytes::{BufMut, Bytes, BytesMut};

pub struct Compiler {
    constants: Vec<BaseObject>,
    scopes: ScopeStack,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            constants: Vec::new(),
            scopes: ScopeStack::new(),
        }
    }

    pub fn compile_program(mut self, stmts: StmtList) -> Bytecode {
        self.compile_stmt_list(stmts);
        self.finish()
    }

    pub fn finish(self) -> Bytecode {
        let (instructions, global_count) = self.scopes.final_scope();
        Bytecode {
            instructions,
            constants: self.constants,
            global_count,
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
                        let sym = self.scopes.register_sym(ident);
                        let code = match sym.scope {
                            ScopeKind::GLOBAL => op::SET_GLOBAL,
                            ScopeKind::LOCAL => op::SET_LOCAL,
                            ScopeKind::LOCKED => unreachable!(),
                        };
                        self.emit_with_u16(code, sym.index as u16);
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
                let sym = self
                    .scopes
                    .lookup_sym(&name)
                    .unwrap_or_else(|| panic!("Symbol \"{name}\" was never declared"));
                self.load_symbol_on_stack(sym);
            }
            Expr::Function { req_params, opt_params, eval } => {
                self.scopes.enter_scope();

                let req_count = req_params.len();
                let opt_count = opt_params.len();
                for param in req_params.into_iter() { self.scopes.register_sym(param); };
                for param in opt_params.into_iter() { self.scopes.register_sym(param); };

                self.compile_expr(*eval);
                self.emit(op::RETURN);

                let (ins, symbol_count, locked_symbols) = self.scopes.exit_scope();

                let locked_sym_count = locked_symbols.len();
                for sym in locked_symbols {
                    self.load_symbol_on_stack(sym);
                }

                let const_ptr = self.add_const(BaseObject::Closure(Function {
                    ins,
                    num_locals: symbol_count,
                    num_req_params: req_count,
                    num_opt_params: opt_count,
                    locked_values: Vec::new(),
                }));

                self.emit_with_u16_u16(op::MAKE_FN, const_ptr, locked_sym_count as u16);
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
            Expr::Postfix { lhs, postfix } => {
                match postfix {
                    Postfix::Call(args) => self.compile_postfix_call(*lhs, args),
                    _ => todo!()
                }
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
            }
            Former::Range(range) => {
                let mut flag = flag_base;
                if range.inclusive {
                    flag |= INCL_BIT
                };
                if let Some(step) = range.step {
                    self.compile_expr(*step);
                    flag |= STEP_BIT;
                };

                self.compile_expr(*range.start);
                self.compile_expr(*range.end);
                self.emit_with_u8(op::MAKE_RN_COL, flag);
            }
            Former::Iterator {
                output: _,
                iterator: _,
            } => unimplemented!(),
        }
    }

    fn compile_postfix_call(&mut self, lhs: Expr, args: ExprList) {
        self.compile_expr(lhs);
        let arg_count = args.len();
        for arg in args { self.compile_expr(arg) };
        self.emit_with_u16(op::CALL, arg_count as u16);
    }

    fn load_symbol_on_stack(&mut self, sym: SymbolRef) {
        let code = match sym.scope {
            ScopeKind::GLOBAL => op::GET_GLOBAL,
            ScopeKind::LOCAL => op::GET_LOCAL,
            ScopeKind::LOCKED => op::GET_LOCKED,
        };
        self.emit_with_u16(code, sym.index as u16);
    }

    /* Buncha convenience methods for modifying the instructions */

    fn last_ins(&mut self) -> &mut BytesMut {
        self.scopes.last_ins_mut()
    }

    fn emit(&mut self, code: Op) {
        self.last_ins().put_u8(code);
    }

    fn emit_with_u8(&mut self, code: Op, operand: u8) {
        self.emit(code);
        self.last_ins().put_u8(operand);
    }

    fn emit_with_u8_u16(&mut self, code: Op, operand_1: u8, operand_2: u16) {
        self.emit(code);
        self.last_ins().put_u8(operand_1);
        self.last_ins().put_u16(operand_2);
    }

    fn emit_with_u16_u16(&mut self, code: Op, operand_1: u16, operand_2: u16) {
        self.emit(code);
        self.last_ins().put_u16(operand_1);
        self.last_ins().put_u16(operand_2);
    }

    fn emit_with_u16(&mut self, code: Op, operand: u16) {
        self.emit(code);
        self.last_ins().put_u16(operand);
    }

    fn emit_with_u32(&mut self, code: Op, operand: u32) {
        self.emit(code);
        self.last_ins().put_u32(operand);
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
        self.scopes.size()
    }

    fn overwrite(&mut self, at: usize, data: Bytes) {
        for (i, byte) in data.into_iter().enumerate() {
            self.last_ins()[at + i] = byte;
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

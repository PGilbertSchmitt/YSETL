use std::collections::HashMap;
use std::rc::Rc;
use std::u32;

use super::bytecode::flags::{
    INCL_BIT, MAP_BASE, RED_BASE, RED_OP_BIT, SET_BASE, STEP_BIT, TUP_BASE,
};
use super::bytecode::Bytecode;
use super::scope::{ScopeKind, ScopeStack, SymbolRef};

use crate::object::object::{Atom, Executor, Object};
use crate::op::{self, Op};
use crate::parser::ast::{
    BinOp, Bound, Expr, ExprList, Former, Iterator, KeyValuePair, MapFormer, Postfix, PreOp,
    SelectOp, SingleIterator, Stmt, StmtList, StmtListWithCapture, SwitchCase,
};
use bytes::{BufMut, Bytes, BytesMut};

pub struct Compiler {
    constants: Vec<Object>,
    named_atoms: HashMap<String, u32>,
    iterators: Vec<Executor>,
    scopes: ScopeStack,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            constants: Vec::new(),
            named_atoms: HashMap::new(),
            iterators: Vec::new(),
            scopes: ScopeStack::new(),
        }
    }

    pub fn compile_program(mut self, stmts: StmtList) -> Bytecode {
        self.compile_stmt_list(stmts);
        self.finish()
    }

    pub fn finish(self) -> Bytecode {
        let (instructions, global_count) = self.scopes.final_scope();

        let mut atoms: Vec<Atom> = self
            .named_atoms
            .into_iter()
            .map(|(s, v)| Atom::new(v, s))
            .collect();
        atoms.sort_by_key(|a| a.0);

        Bytecode {
            instructions,
            constants: self.constants,
            iterators: self.iterators,
            global_count,
            atoms,
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
            Stmt::PrintDbg(expr) => {
                self.compile_expr(expr);
                self.emit(op::PRINT_DBG); // Performs a pop
            }
            Stmt::Assign { target, value } => {
                match target {
                    Bound::Ident(ident) => {
                        let sym = self.scopes.register_sym(ident);
                        self.compile_expr(value);
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
            Stmt::Return(expr) => self.compile_return(expr),
        }
    }

    // Should leave something on the stack, which may be used by conditional expressions
    // and function bodies. Non-expression statements should leave a `null` on the stack.
    fn compile_stmt_like_expr(&mut self, node: Stmt) {
        match node {
            Stmt::Expr(expr) => self.compile_expr(expr),
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.emit(op::PRINT);
                self.emit(op::NULL);
            }
            Stmt::PrintDbg(expr) => {
                self.compile_expr(expr);
                self.emit(op::PRINT_DBG);
                self.emit(op::NULL);
            }
            Stmt::Assign { .. } => panic!("Cannot assign a variable here!"),
            // It doesn't matter what's left on the stack here because returning
            // will reset the stack anyways.
            Stmt::Return(expr) => self.compile_return(expr),
        }
    }

    fn compile_expr(&mut self, node: Expr) {
        match node {
            Expr::Null => self.emit(op::NULL),
            Expr::True => self.emit(op::TRUE),
            Expr::False => self.emit(op::FALSE),
            Expr::Newat => self.emit(op::MAKE_ATOM),
            Expr::Integer(value) => {
                let const_ptr = self.add_const(Object::Int(value));
                self.emit_with_u16(op::CONST, const_ptr);
            }
            Expr::Float(value) => {
                let const_ptr = self.add_const(Object::Float(value));
                self.emit_with_u16(op::CONST, const_ptr);
            }
            Expr::Atom(name) => {
                let atom_count = self.named_atoms.len() as u32;
                let atom_ptr = *self.named_atoms.entry(name).or_insert(atom_count);
                self.emit_with_u32(op::GET_ATOM, atom_ptr);
            }
            Expr::String(value) => {
                let const_ptr = self.add_const(Object::new_string(value));
                self.emit_with_u16(op::CONST, const_ptr);
            }
            Expr::Map(former) => self.compile_map_former(former),
            Expr::Tuple(former) => self.compile_former(former, TUP_BASE),
            Expr::Set(former) => self.compile_former(former, SET_BASE),
            Expr::Ident(name) => {
                let sym = self
                    .scopes
                    .lookup_sym(&name)
                    .unwrap_or_else(|| panic!("Symbol \"{name}\" was never declared"));
                self.load_symbol_on_stack(sym);
            }
            Expr::Function {
                req_params,
                opt_params,
                eval,
            } => {
                self.scopes.enter_scope();

                let req_count = req_params.len();
                let opt_count = opt_params.len();
                for param in req_params.into_iter() {
                    self.scopes.register_sym(param);
                }
                for param in opt_params.into_iter() {
                    self.scopes.register_sym(param);
                }

                self.compile_expr(*eval);
                // TODO: This return is only necessary for implicit-return functions
                // Block-expression based functions will generate their own returns when implemented
                // in the compiler
                self.emit(op::RETURN);

                let (ins, symbol_count, locked_symbols) = self.scopes.exit_scope();

                let locked_sym_count = locked_symbols.len();
                for sym in locked_symbols {
                    self.load_symbol_on_stack(sym);
                }

                let const_ptr = self.add_const(Object::new_closure(
                    Executor {
                        ins,
                        num_locals: symbol_count,
                        locked_values: Rc::new(Vec::new()),
                    },
                    req_count,
                    opt_count,
                ));

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
            Expr::Postfix { lhs, postfix } => match postfix {
                Postfix::Call(args) => self.compile_postfix_call(*lhs, args),
                _ => todo!(),
            },
            Expr::Ternary {
                condition,
                consequence,
                alternative,
            } => {
                self.compile_expr(*condition);

                let jump_if_operand_ptr = self.ins_len() + 1;
                self.emit_with_u32(op::JUMP_IF_FALSE, u32::MAX);
                self.compile_stmt_like_expr(*consequence);

                let jump_operand_ptr = self.ins_len() + 1;
                self.emit_with_u32(op::JUMP, u32::MAX);
                let jnt_destination = self.ins_len();
                self.compile_stmt_like_expr(*alternative);
                let jump_destination = self.ins_len();

                // Set correct jump locations
                self.overwrite_u32(jump_if_operand_ptr, jnt_destination as u32);
                self.overwrite_u32(jump_operand_ptr, jump_destination as u32)
            }
            Expr::Select { op, iterator } => self.compile_select_iterator(op, iterator),
            Expr::Switch { condition, cases } => {
                let is_match_switch = condition.is_some();
                let conditional_jump_operator = if is_match_switch {
                    op::JUMP_NOT_MATCH
                } else {
                    op::JUMP_IF_FALSE
                };

                condition.map(|match_expr| {
                    self.compile_expr(*match_expr);
                    self.emit(op::PUSH_MATCH);
                });

                let mut last_cond_jump_ptr: Option<usize> = None;
                let mut unconditional_jump_ptrs: Vec<usize> = Vec::with_capacity(cases.len());
                let mut has_default_case = false;

                for SwitchCase {
                    condition,
                    consequence,
                } in cases
                {
                    if let Some(ptr_location) = last_cond_jump_ptr {
                        self.overwrite_u32(ptr_location, self.ins_len() as u32);
                    }
                    match condition {
                        Some(expr) => {
                            self.compile_expr(expr);

                            let cond_jump_ptr = self.ins_len() + 1;
                            last_cond_jump_ptr = Some(cond_jump_ptr);
                            self.emit_with_u32(conditional_jump_operator, u32::MAX);
                            self.compile_stmt_like_expr(consequence);

                            let uncond_jump_ptr = self.ins_len() + 1;
                            unconditional_jump_ptrs.push(uncond_jump_ptr);
                            self.emit_with_u32(op::JUMP, u32::MAX);
                        }
                        None => {
                            has_default_case = true;
                            self.compile_stmt_like_expr(consequence);
                            break;
                        }
                    }
                }

                if !has_default_case {
                    if let Some(ptr_location) = last_cond_jump_ptr {
                        self.overwrite_u32(ptr_location, self.ins_len() as u32);
                    }
                    self.emit(op::NULL);
                }

                if is_match_switch {
                    self.emit(op::POP_MATCH);
                };

                let continue_ptr = self.ins_len() as u32;
                unconditional_jump_ptrs.into_iter().for_each(|jump_ptr| {
                    self.overwrite_u32(jump_ptr, continue_ptr);
                });
            }
            // A block is basically just a function with 0 parameters that is executed immediately, but I
            // feel like there should be a more efficient way of doing this. It's not coming to me
            // immediately, but perhaps I can at least combine a real function with it's block expr
            // into a single function. That shuffling act should be much simpler to solve.
            Expr::Block(StmtListWithCapture {
                stmt_list,
                implicit_return,
            }) => {
                self.scopes.enter_scope();
                stmt_list
                    .into_iter()
                    .for_each(|stmt| self.compile_stmt(stmt));
                if let Some(stmt) = implicit_return {
                    self.compile_stmt_like_expr(*stmt);
                } else {
                    self.emit(op::NULL);
                }
                self.emit(op::RETURN);
                let (ins, symbol_count, locked_symbols) = self.scopes.exit_scope();

                let locked_sym_count = locked_symbols.len();
                for sym in locked_symbols {
                    self.load_symbol_on_stack(sym);
                }

                let const_ptr = self.add_const(Object::new_closure(
                    Executor {
                        ins,
                        num_locals: symbol_count,
                        locked_values: Rc::new(Vec::new()),
                    },
                    0,
                    0,
                ));

                // This is the part I don't like
                self.emit_with_u16_u16(op::MAKE_FN, const_ptr, locked_sym_count as u16);
                self.emit_with_u16(op::CALL, 0);
            }
            Expr::ReduceOp { op, lhs, rhs } => {
                self.compile_expr(*lhs); // Initial accumulator left on stack
                self.compile_expr(*rhs); // Single iteration collection left on stack

                // TODO: Basically everything from here can be converted into a pre-compiled
                //iterator, since the inside is always the same thing.
                self.scopes.enter_scope();

                // The indexes and jump pointers are always the same
                self.emit_with_u8(op::MAKE_ITER, 0);
                self.emit_with_u32(op::ITER_EMPTY_CHECK, 17);
                self.emit(op::GET_ACC);
                self.emit_with_u8(op::GET_ITER_VAL, 0);
                // An operator as an operand!?
                self.emit_with_u8(op::REDUCE_WITH, from_binop(op));
                self.emit_with_u8_u32(op::ITER_NEXT, 0, 7);
                self.emit(op::ITER_END);

                let (ins, _, _) = self.scopes.exit_scope();
                let global_iter_idx = self.iterators.len() as u16;
                self.iterators.push(Executor {
                    ins,
                    num_locals: 0,
                    locked_values: Rc::new(Vec::new()),
                });
                self.emit_with_u16_u16_u8_u8(
                    op::ITER_START,
                    global_iter_idx,
                    0,
                    1,
                    RED_BASE | RED_OP_BIT,
                );
            }
            Expr::ReduceExpr { reducer, lhs, rhs } => {
                self.compile_expr(*reducer); // Reducer left on stack
                self.compile_expr(*lhs); // Initial accumulator left on stack
                self.compile_expr(*rhs); // Single iteration collection left on stack

                // TODO: Basically everything from here can be converted into a pre-compiled
                //iterator, since the inside is always the same thing.
                self.scopes.enter_scope();

                // The indexes and jump pointers are always the same
                self.emit_with_u8(op::MAKE_ITER, 0);
                self.emit_with_u32(op::ITER_EMPTY_CHECK, 17);
                self.emit(op::GET_ACC);
                self.emit_with_u8(op::GET_ITER_VAL, 0);
                self.emit(op::REDUCE_CALL);
                self.emit(op::ITER_COLLECT);
                self.emit_with_u8_u32(op::ITER_NEXT, 0, 7);
                self.emit(op::ITER_END);

                let (ins, _, _) = self.scopes.exit_scope();
                let global_iter_idx = self.iterators.len() as u16;
                self.iterators.push(Executor {
                    ins,
                    num_locals: 0,
                    locked_values: Rc::new(Vec::new()),
                });
                self.emit_with_u16_u16_u8_u8(op::ITER_START, global_iter_idx, 0, 1, RED_BASE);
            }
            Expr::Inject { injector, lhs, rhs } => {
                self.compile_expr(*injector);
                self.compile_expr(*lhs);
                self.compile_expr(*rhs);
                self.emit_with_u16(op::CALL, 2);
            }
            Expr::Insert { key, lhs, rhs } => {
                self.compile_expr(*lhs); // Map
                self.compile_expr(*key); // Key
                self.compile_expr(*rhs); // Value
                self.emit(op::INSERT);
            }
        };
    }

    fn compile_return(&mut self, expr: Option<Expr>) {
        if let Some(expr) = expr {
            self.compile_expr(expr);
        } else {
            self.emit(op::NULL);
        }
        self.emit(op::RETURN);
    }

    fn compile_binary_op_with_jump(&mut self, op: Op, lhs: Box<Expr>, rhs: Box<Expr>) {
        self.compile_expr(*lhs);
        let jump_operand_ptr = self.ins_len() + 1;
        self.emit_with_u32(op, u32::MAX);
        self.compile_expr(*rhs);
        let jump_destination = self.ins_len();
        self.overwrite_u32(jump_operand_ptr, jump_destination as u32);
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
            Former::Iterator { output, iterator } => {
                self.compile_iterator_former(*output, iterator, flag_base)
            }
        }
    }

    fn compile_map_former(&mut self, former: MapFormer) {
        match former {
            MapFormer::Empty => self.emit_with_u8_u16(op::MAKE_LIT_COL, MAP_BASE, 0),
            MapFormer::Literal(pairs) => {
                let size = pairs.len() as u16;
                for KeyValuePair(key, value) in pairs.into_iter() {
                    self.compile_expr(key);
                    self.compile_expr(value);
                }
                self.emit_with_u8_u16(op::MAKE_LIT_COL, MAP_BASE, size);
            }
            MapFormer::Iterator {
                key_output,
                value_output,
                iterator,
            } => self.compile_map_iterator_former(*key_output, *value_output, iterator),
        }
    }

    fn compile_iterator_symbol_loader(&mut self, iter_vars: &Vec<IterVar>) -> usize {
        let iteration_start_ptr = self.ins_len();

        // Using this `sym_count` assumes that the symbols were registered in the correct order
        let mut sym_count: u16 = 0;
        for (idx, iter_var) in iter_vars.iter().enumerate() {
            let idx = idx as u8;
            match iter_var {
                IterVar::KeyAndValue => {
                    self.emit_with_u8(op::GET_ITER_VAL, idx);
                    self.emit_with_u16(op::SET_LOCAL, sym_count);
                    sym_count += 1;
                    self.emit_with_u8(op::GET_ITER_KEY, idx);
                    self.emit_with_u16(op::SET_LOCAL, sym_count);
                    sym_count += 1;
                }
                IterVar::Value => {
                    self.emit_with_u8(op::GET_ITER_VAL, idx);
                    self.emit_with_u16(op::SET_LOCAL, sym_count);
                    sym_count += 1;
                }
            }
        }

        iteration_start_ptr
    }

    fn compile_iterator_filter(&mut self, filter: Option<Box<Expr>>, jump_op: Op) -> Option<usize> {
        filter.map(|filter| {
            self.compile_expr(*filter);
            let dest = self.ins_len() + 1;
            self.emit_with_u32(jump_op, u32::MAX);
            dest
        })
    }

    fn compile_iterator_nexts(
        &mut self,
        iter_vars: &Vec<IterVar>,
        iteration_start_ptr: u32,
        empty_check_ptr: usize,
    ) {
        for (idx, _) in iter_vars.iter().enumerate().rev() {
            self.emit_with_u8_u32(op::ITER_NEXT, idx as u8, iteration_start_ptr);
        }
        self.overwrite_u32(empty_check_ptr, self.ins_len() as u32);
    }

    fn compile_iterator_start(
        &mut self,
        iterator: Iterator,
    ) -> (Vec<IterVar>, Option<Box<Expr>>, usize, usize, u8) {
        let Iterator { iterators, filter } = iterator;

        let iterator_count = iterators.len();
        if iterator_count > 255 {
            panic!("Cannot support an iterator with more than 255 members")
        };

        // In the scope before we initiate the iterator, we can compile the collections
        // used for the iterators:

        // This line is kinda funny if you think about it, and also a nightmare. We need to iterate
        // over the iterators and evaluate them onto the stack BEFORE entering a new scope. We need
        // ownership of the expressions to iterator them, but we'll need to iterate again inside
        // the new scope with the bounds, so we collect those.
        let iters_only_bounds: Vec<_> = iterators
            .into_iter()
            .map(|single_iter| match single_iter {
                SingleIterator::In { expr, bounds } => {
                    self.compile_expr(expr);
                    SingleIterator::In {
                        bounds,
                        expr: Expr::Null,
                    }
                }
                SingleIterator::Select {
                    collection,
                    key,
                    value,
                    ..
                } => {
                    let collection_sym = self
                        .scopes
                        .lookup_sym(&collection)
                        .expect("Key-Value iterator must be an initialized variable");
                    self.load_symbol_on_stack(collection_sym);
                    SingleIterator::Select {
                        collection: String::new(),
                        key,
                        value,
                    }
                }
            })
            .collect();

        self.scopes.enter_scope();

        // Now we need to iterate over the iterators again AFTER we enter a new scope so that
        // we can register the bounds where they will be evaluated.
        let mut iter_vars: Vec<IterVar> = vec![];
        iters_only_bounds
            .into_iter()
            .enumerate()
            .for_each(|(i, single_iter)| match single_iter {
                SingleIterator::In { bounds, .. } => {
                    if bounds.len() > u8::MAX as usize {
                        panic!("Cannot support an iterator with more than 255 bounds");
                    }
                    for bound in bounds.into_iter() {
                        self.emit_with_u8(op::MAKE_ITER, i as u8);
                        self.register_bound(bound);
                        iter_vars.push(IterVar::Value);
                    }
                }
                SingleIterator::Select { key, value, .. } => {
                    self.emit_with_u8(op::MAKE_ITER, i as u8);
                    self.register_bound(value);
                    self.register_bound(key);
                    iter_vars.push(IterVar::KeyAndValue);
                }
            });

        // We only need to do this once to ensure we don't start iterating if any of the
        // collections are empty to begin with. After iteration starts, the `ITER_NEXT`
        // opcodes are responsible for handling their own emptiness (aren't we all?)
        let empty_check_ptr = self.ins_len() + 1;
        self.emit_with_u32(op::ITER_EMPTY_CHECK, u32::MAX);
        let iteration_start_ptr = self.compile_iterator_symbol_loader(&iter_vars);

        (
            iter_vars,
            filter,
            empty_check_ptr,
            iteration_start_ptr,
            iterator_count as u8,
        )
    }

    fn compile_iterator_end(&mut self, collection_count: u8, flag_base: u8) {
        let (ins, symbol_count, locked_symbols) = self.scopes.exit_scope();

        let locked_sym_count = locked_symbols.len() as u16;
        for sym in locked_symbols {
            self.load_symbol_on_stack(sym);
        }

        let global_iter_idx = self.iterators.len() as u16;
        self.iterators.push(Executor {
            ins,
            num_locals: symbol_count,
            locked_values: Rc::new(Vec::new()),
        });

        self.emit_with_u16_u16_u8_u8(
            op::ITER_START,
            global_iter_idx,
            locked_sym_count,
            collection_count,
            flag_base,
        );
    }

    fn compile_select_iterator(&mut self, select_op: SelectOp, iterator: Iterator) {
        let (iter_vars, filter, empty_check_ptr, iteration_start_ptr, collection_count) =
            self.compile_iterator_start(iterator);
        let jump_ptr_dest = self.compile_iterator_filter(
            filter,
            if select_op == SelectOp::FORALL {
                op::JUMP_IF_TRUE
            } else {
                op::JUMP_IF_FALSE
            },
        );

        /* Early returns */
        match select_op {
            SelectOp::EXISTS => {
                self.emit(op::TRUE);
                self.emit(op::RETURN);
            }
            SelectOp::FORALL => {
                self.emit(op::FALSE);
                self.emit(op::RETURN);
            }
            SelectOp::CHOOSE => {
                let sub_iter_count = iter_vars.len();

                iter_vars
                    .iter()
                    .enumerate()
                    .for_each(|(idx, iter_var)| match iter_var {
                        IterVar::Value => {
                            self.emit_with_u8(op::GET_ITER_VAL, idx as u8);
                        }
                        IterVar::KeyAndValue => {
                            self.emit_with_u8(op::GET_ITER_VAL, idx as u8);
                            self.emit_with_u8(op::GET_ITER_KEY, idx as u8);
                            self.emit_with_u8_u16(op::MAKE_LIT_COL, TUP_BASE, 2);
                        }
                    });

                if sub_iter_count > 1 {
                    self.emit_with_u8_u16(op::MAKE_LIT_COL, TUP_BASE, sub_iter_count as u16);
                }
                self.emit(op::RETURN);
            }
        }

        // Since we now know where the iterate incrementers start, we can update the
        // pointer of the jump (if it exists)
        if let Some(dest) = jump_ptr_dest {
            self.overwrite_u32(dest as usize, self.ins_len() as u32);
        }
        self.compile_iterator_nexts(&iter_vars, iteration_start_ptr as u32, empty_check_ptr);

        /* Iteration finished */
        match select_op {
            SelectOp::EXISTS => {
                self.emit(op::FALSE);
                self.emit(op::RETURN);
            }
            SelectOp::FORALL => {
                self.emit(op::TRUE);
                self.emit(op::RETURN);
            }
            SelectOp::CHOOSE => {
                self.emit(op::NULL);
                self.emit(op::RETURN);
            }
        }
        self.compile_iterator_end(collection_count, TUP_BASE);
    }

    fn compile_iterator_former(&mut self, eval: Expr, iterator: Iterator, flag_base: u8) {
        let (iter_vars, filter, empty_check_ptr, iteration_start_ptr, collection_count) =
            self.compile_iterator_start(iterator);
        let jump_ptr_dest = self.compile_iterator_filter(filter, op::JUMP_IF_FALSE);

        self.compile_expr(eval);
        self.emit(op::ITER_COLLECT);

        // Since we now know where the iterate incrementers start, we can update the
        // pointer of the jump (if it exists)
        if let Some(dest) = jump_ptr_dest {
            self.overwrite_u32(dest as usize, self.ins_len() as u32);
        }

        self.compile_iterator_nexts(&iter_vars, iteration_start_ptr as u32, empty_check_ptr);
        self.emit(op::ITER_END);
        self.compile_iterator_end(collection_count, flag_base);
    }

    fn compile_map_iterator_former(
        &mut self,
        key_eval: Expr,
        value_eval: Expr,
        iterator: Iterator,
    ) {
        let (iter_vars, filter, empty_check_ptr, iteration_start_ptr, collection_count) =
            self.compile_iterator_start(iterator);
        let jump_ptr_dest = self.compile_iterator_filter(filter, op::JUMP_IF_FALSE);

        self.compile_expr(key_eval);
        self.compile_expr(value_eval);
        self.emit(op::ITER_COLLECT_KV);

        // Since we now know where the iterate incrementers start, we can update the
        // pointer of the jump (if it exists)
        if let Some(dest) = jump_ptr_dest {
            self.overwrite_u32(dest as usize, self.ins_len() as u32);
        }

        self.compile_iterator_nexts(&iter_vars, iteration_start_ptr as u32, empty_check_ptr);
        self.emit(op::ITER_END);
        self.compile_iterator_end(collection_count, MAP_BASE);
    }

    fn register_bound(&mut self, bound: Bound) {
        match bound {
            Bound::Ident(id) => self.scopes.register_sym(id),
            _ => unimplemented!(),
        };
    }

    fn compile_postfix_call(&mut self, lhs: Expr, args: ExprList) {
        self.compile_expr(lhs);
        let arg_count = args.len();
        for arg in args {
            self.compile_expr(arg)
        }
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

    fn emit_with_u8_u32(&mut self, code: Op, operand_1: u8, operand_2: u32) {
        self.emit(code);
        self.last_ins().put_u8(operand_1);
        self.last_ins().put_u32(operand_2);
    }

    fn emit_with_u16_u16(&mut self, code: Op, operand_1: u16, operand_2: u16) {
        self.emit(code);
        self.last_ins().put_u16(operand_1);
        self.last_ins().put_u16(operand_2);
    }

    // I need something better than this...
    fn emit_with_u16_u16_u8_u8(
        &mut self,
        code: Op,
        operand_1: u16,
        operand_2: u16,
        operand_3: u8,
        operand_4: u8,
    ) {
        self.emit(code);
        self.last_ins().put_u16(operand_1);
        self.last_ins().put_u16(operand_2);
        self.last_ins().put_u8(operand_3);
        self.last_ins().put_u8(operand_4);
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

    fn add_const(&mut self, base_object: Object) -> u16 {
        self.constants.push(base_object);
        (self.constants.len() - 1)
            .try_into()
            .expect("Too many constants were generated")
    }

    // fn add_atom(&mut self, )

    fn ins_len(&self) -> usize {
        self.scopes.ins_len()
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

#[derive(Debug)]
enum IterVar {
    Value,
    KeyAndValue,
}

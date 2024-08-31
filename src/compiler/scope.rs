use std::{collections::HashMap, rc::Rc};
use bytes::{Bytes, BytesMut};

#[derive(Debug, PartialEq)]
pub enum ScopeKind {
    GLOBAL,
    LOCAL,
    LOCKED,
}

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub scope: ScopeKind,
    pub index: usize,
    pub name: String,
}

pub type SymbolRef = Rc<Symbol>;

#[derive(Debug)]
pub struct SymbolRegistry {
    table: HashMap<String, SymbolRef>,
    stack_idx: usize,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            stack_idx: 0, // The index for non-locked variables
        }
    }
}

#[derive(Debug)]
pub struct Scope {
    instructions: BytesMut,
    symbols: SymbolRegistry,
    locked: Vec<SymbolRef>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            instructions: BytesMut::new(),
            symbols: SymbolRegistry::new(),
            locked: Vec::new(),
        }
    }

    pub fn push_locked_sym(&mut self, id: String, original: SymbolRef) -> SymbolRef {
        let new_sym = Rc::new(Symbol {
            scope: ScopeKind::LOCKED,
            index: self.locked.len(),
            name: id.to_owned(),
        });
        self.locked.push(original);
        self.symbols.table.insert(id, new_sym.clone());
        new_sym
    }
}

#[derive(Debug)]
pub struct ScopeStack (Vec<Scope>);

impl ScopeStack {
    pub fn new() -> Self {
        Self (vec![Scope::new()])
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn ins_len(&self) -> usize {
        self.0.last().unwrap().instructions.len()
    }

    /** Returns the number of symbols in the current scope */
    pub fn symbol_count(&self) -> usize {
        self.last_symbols().stack_idx
    }

    pub fn enter_scope(&mut self) {
        self.0.push(Scope::new())
    }

    pub fn exit_scope(&mut self) -> (Bytes, usize, Vec<SymbolRef>) {
        if self.size() < 2 {
            panic!("Expected there to be more than 1 scope, but ended up with {}. \
            Scoping was not handled properly by compiler", self.size());
        } 
        let scope = self.0.pop().unwrap();
        (scope.instructions.freeze(), scope.symbols.stack_idx, scope.locked)
    }

    pub fn final_scope(self) -> (Bytes, usize) {
        if self.size() != 1 {
            panic!("Expected there to be exactly 1 scope, but compilation ended \
            with {}. Scoping was not handled properly by compiler", self.size());
        }
        let scope = self.0.into_iter().next().unwrap();
        (scope.instructions.freeze(), scope.symbols.stack_idx)
    }

    pub fn register_sym(&mut self, id: String) -> SymbolRef {
        let index = self.symbol_count();
        let scope = if self.size() == 1 {
            ScopeKind::GLOBAL
        } else {
            ScopeKind::LOCAL
        };
        let registry = self.last_symbols_mut();

        registry.table
            .entry(id.to_owned())
            .or_insert_with(|| {
                registry.stack_idx += 1;
                Rc::new(Symbol { scope, index, name: id })
            })
            .clone()
    }

    pub fn lookup_sym(&mut self, id: &str) -> Option<SymbolRef> {
        let mut scopes = self.0.iter().enumerate().rev();
        let (_, cur_scope) = scopes.next().unwrap();
        if let Some(sym) = cur_scope.symbols.table.get(id) {
            // Variable is local to scope
            return Some(sym.clone());
        };

        let (idx, sym) = scopes.find_map(|(i, scope)| {
            scope.symbols.table.get(id).map(|sym| (i+1, sym.clone()))
        })?;

        if sym.scope == ScopeKind::GLOBAL {
            return Some(sym);
        }

        // The idx indicates the index of the first scope that needs to have the locked
        // variable registered. This would be less complicated if I implemented the
        // scope stack recursively. This emulates the recursion by bubbling the new
        // symbol up the stack.
        let mut last = sym;
        for scope in self.0[idx..].iter_mut() {
            last = scope.push_locked_sym(id.to_owned(), last.clone());
        }

        Some(last)
    }

    fn last_symbols(&self) -> &SymbolRegistry {
        &self.0.last().unwrap().symbols
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        // Unwrap is safe as long as the only way to remove scopes is through the `exit_scope`
        // method, which ensures that there is always at least 1 scope in the tank at all times.
        self.0.last_mut().unwrap()
    }

    fn last_symbols_mut(&mut self) -> &mut SymbolRegistry {
        &mut self.current_scope_mut().symbols
    }

    pub fn last_ins_mut(&mut self) -> &mut BytesMut {
        &mut self.current_scope_mut().instructions
    }

    pub fn peek_ins(&self) -> Bytes {
        self.0.last().unwrap().instructions.clone().freeze()
    }

    pub fn peek_symbols(&self) -> &SymbolRegistry {
        &self.0.last().unwrap().symbols
    }
}

#[cfg(test)]
mod tests {
    use super::ScopeKind;
    use super::ScopeStack;

    #[test]
    fn globals() {
        let mut scopes = ScopeStack::new();
        scopes.register_sym("a".to_owned());
        scopes.register_sym("b".to_owned());
        assert_eq!(scopes.symbol_count(), 2);

        let symbol_a = scopes.lookup_sym("a").unwrap();
        assert_eq!(symbol_a.index, 0);
        assert_eq!(symbol_a.scope, ScopeKind::GLOBAL);

        let symbol_b = scopes.lookup_sym("b").unwrap();
        assert_eq!(symbol_b.index, 1);
        assert_eq!(symbol_b.scope, ScopeKind::GLOBAL);

        assert!(scopes.lookup_sym("c").is_none());
    }

    #[test]
    fn globals_in_local_scope() {
        let mut scopes = ScopeStack::new();
        scopes.register_sym("a".to_owned());
        scopes.register_sym("b".to_owned());
        scopes.enter_scope();
        scopes.register_sym("b".to_owned());

        let symbol_a = scopes.lookup_sym("a").unwrap();
        assert_eq!(symbol_a.index, 0);
        assert_eq!(symbol_a.scope, ScopeKind::GLOBAL);
    }

    #[test]
    fn locals() {
        let mut scopes = ScopeStack::new();
        scopes.register_sym("a".to_owned());
        scopes.register_sym("b".to_owned());
        scopes.enter_scope();
        scopes.register_sym("b".to_owned());

        let local_symbol_b = scopes.lookup_sym("b").unwrap();
        assert_eq!(local_symbol_b.index, 0);
        assert_eq!(local_symbol_b.scope, ScopeKind::LOCAL);
    }

    #[test]
    fn locals_after_leaving_scope() {
        let mut scopes = ScopeStack::new();
        scopes.register_sym("a".to_owned());
        scopes.enter_scope();
        scopes.register_sym("b".to_owned());
        scopes.exit_scope();

        assert!(scopes.lookup_sym("b").is_none())
    }

    #[test]
    fn reregister_symbol_in_same_scope() {
        let mut scopes = ScopeStack::new();
        scopes.register_sym("a".to_owned());
        assert_eq!(scopes.lookup_sym("a").unwrap().index, 0);
        scopes.register_sym("a".to_owned());
        assert_eq!(scopes.lookup_sym("a").unwrap().index, 0);

        scopes.enter_scope();
        scopes.register_sym("b".to_owned());
        assert_eq!(scopes.lookup_sym("b").unwrap().index, 0);
        scopes.register_sym("b".to_owned());
        assert_eq!(scopes.lookup_sym("b").unwrap().index, 0);
    }

    #[test]
    fn locked_symbols() {
        let mut scopes = ScopeStack::new();
        scopes.register_sym("a".to_owned());
        scopes.enter_scope();
        scopes.register_sym("b".to_owned());
        scopes.enter_scope();
        scopes.register_sym("c".to_owned());
        scopes.enter_scope();
        scopes.register_sym("d".to_owned());

        assert_eq!(scopes.lookup_sym("e"), None);
        assert_eq!(scopes.lookup_sym("d").unwrap().scope, ScopeKind::LOCAL);
        assert_eq!(scopes.lookup_sym("c").unwrap().scope, ScopeKind::LOCKED);
        assert_eq!(scopes.lookup_sym("b").unwrap().scope, ScopeKind::LOCKED);
        assert_eq!(scopes.lookup_sym("a").unwrap().scope, ScopeKind::GLOBAL);
    }
}

use std::{collections::HashMap, rc::Rc, vec};

#[derive(Debug, PartialEq)]
pub enum Scope {
    GLOBAL,
    LOCAL,
}

#[derive(Debug, PartialEq)]
pub struct Symbol {
    pub scope: Scope,
    pub index: u16,
}

type SymbolRegistry = HashMap<String, Rc<Symbol>>;

pub struct SymbolStack {
    registries: Vec<SymbolRegistry>,
}

impl SymbolStack {
    pub fn new() -> Self {
        Self {
            registries: vec![SymbolRegistry::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.registries.push(SymbolRegistry::new());
    }

    pub fn exit_scope(&mut self) {
        if self.registries.len() == 1 {
            panic!("Cannot pop last scope");
        }
        self.registries.pop().unwrap();
    }

    pub fn register_sym(&mut self, id: String) -> Rc<Symbol> {
        let index = self.size();
        let scope = if self.registries.len() == 1 {
            Scope::GLOBAL
        } else {
            Scope::LOCAL
        };
        self.last_mut()
            .entry(id)
            .or_insert_with(|| Rc::new(Symbol { scope, index }))
            .clone()
    }

    pub fn lookup(&self, id: &str) -> Option<Rc<Symbol>> {
        self.registries.iter().rev().find_map(|reg| {
            reg.get(id).map(Rc::clone)
        })
    }

    pub fn size(&self) -> u16 {
        self.last().len() as u16
    }

    fn last(&self) -> &SymbolRegistry {
        self.registries.last().expect("No scopes remaining")
    }

    fn last_mut(&mut self) -> &mut SymbolRegistry {
        self.registries.last_mut().expect("No scopes remaining")
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;
    use super::SymbolStack;

    #[test]
    fn globals() {
        let mut reg = SymbolStack::new();
        reg.register_sym(String::from("a"));
        reg.register_sym(String::from("b"));
        assert_eq!(reg.size(), 2);

        let symbol_a = reg.lookup("a").unwrap();
        assert_eq!(symbol_a.index, 0);
        assert_eq!(symbol_a.scope, Scope::GLOBAL);
        
        let symbol_b = reg.lookup("b").unwrap();
        assert_eq!(symbol_b.index, 1);
        assert_eq!(symbol_b.scope, Scope::GLOBAL);

        assert!(reg.lookup("c").is_none());
    }

    #[test]
    fn globals_in_local_scope() {
        let mut reg = SymbolStack::new();
        reg.register_sym(String::from("a"));
        reg.register_sym(String::from("b"));
        reg.enter_scope();
        reg.register_sym(String::from("b"));

        let symbol_a = reg.lookup("a").unwrap();
        assert_eq!(symbol_a.index, 0);
        assert_eq!(symbol_a.scope, Scope::GLOBAL);
    }

    #[test]
    fn locals() {
        let mut reg = SymbolStack::new();
        reg.register_sym(String::from("a"));
        reg.register_sym(String::from("b"));
        reg.enter_scope();
        reg.register_sym(String::from("b"));

        let local_symbol_b = reg.lookup("b").unwrap();
        assert_eq!(local_symbol_b.index, 0);
        assert_eq!(local_symbol_b.scope, Scope::LOCAL);
    }

    #[test]
    fn locals_after_leaving_scope() {
        let mut reg = SymbolStack::new();
        reg.register_sym(String::from("a"));
        reg.enter_scope();
        reg.register_sym(String::from("b"));
        reg.exit_scope();

        assert!(reg.lookup("b").is_none())
    }
}

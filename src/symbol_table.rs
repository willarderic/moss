use crate::ast::VariableType;

use std::collections::HashMap;

enum Symbol {
    VARIABLE(VariableType),
    TYPE,
}

pub struct SymbolTable {
   table: HashMap<String, Symbol> 
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    // pub fn define(&mut self, identifier: &str, symbol: Symbol) {
    //     self.table.insert(String::from(identifier), symbol); 
    // }

    pub fn define_variable(&mut self, identifier: &str, var_type: &VariableType) {
        self.table.insert(String::from(identifier), Symbol::VARIABLE(var_type.clone()));
    }

    // pub fn get(&self, identifier: &str) -> Option<&Symbol> {
    //     self.table.get(identifier)
    // }

    pub fn get_variable(&self, identifier: &str) -> Option<&VariableType> {
        match self.table.get(identifier) {
            Some(symbol) => match symbol {
                Symbol::VARIABLE(var_type) => Some(&var_type),
                _ => None
            }
            _ => None
        }
    }
}


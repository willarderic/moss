use std::fmt::{Display, Formatter, Result};
use crate::ast::VariableType;

use std::collections::HashMap;

pub struct VariableInfo {
    pub id: u64,
    pub var_type: VariableType,
}

enum Symbol {
    VARIABLE(VariableInfo),
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

    pub fn define_variable(&mut self, identifier: &str, var_info: VariableInfo) {
        self.table.insert(String::from(identifier), Symbol::VARIABLE(var_info));
    }

    // pub fn get(&self, identifier: &str) -> Option<&Symbol> {
    //     self.table.get(identifier)
    // }

    pub fn get_variable(&self, identifier: &str) -> Option<&VariableInfo> {
        match self.table.get(identifier) {
            Some(symbol) => match symbol {
                Symbol::VARIABLE(var_info) => Some(&var_info),
                _ => None
            }
            _ => None
        }
    }
}

impl Display for SymbolTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "SymbolTable:\n").unwrap();
        for key in self.table.keys() {
            match &self.table[key] {
                Symbol::VARIABLE(var_info) => write!(f, "name: [{} | id: {} | {}]\n", key, var_info.id, var_info.var_type).unwrap(),
                Symbol::TYPE => write!(f, "{}", key).unwrap(),
            };
        };
        Ok(())
    }
}

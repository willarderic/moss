use std::fmt::{Display, Formatter, Result};
use crate::ast::VariableType;

use std::collections::HashMap;

#[derive(Clone)]
pub struct VariableInfo {
    pub id: u64,
    pub var_type: VariableType,
}

pub struct TypeInfo {
    pub size: u64,
}

enum Symbol {
    VARIABLE(VariableInfo),
    TYPE(TypeInfo),
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

    pub fn define_variable(&mut self, ident: &str, var_info: VariableInfo) {
        self.table.insert(String::from(ident), Symbol::VARIABLE(var_info));
    }

    pub fn get_variable(&self, ident: &str) -> Option<&VariableInfo> {
        match self.table.get(ident) {
            Some(symbol) => match symbol {
                Symbol::VARIABLE(var_info) => Some(&var_info),
                _ => None
            }
            _ => None
        }
    }
    
    pub fn define_type(&mut self, ident: &str, type_info: TypeInfo) {
        self.table.insert(String::from(ident), Symbol::TYPE(type_info));
    }


    pub fn get_type(&self, ident: &str) -> Option<&TypeInfo> {
        match self.table.get(ident) {
            Some(symbol) => match symbol {
                Symbol::TYPE(type_info) => Some(&type_info),
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
                Symbol::TYPE(type_info) => write!(f, "[type: {} | size: {}]\n", key, type_info.size).unwrap(),
            };
        };
        Ok(())
    }
}

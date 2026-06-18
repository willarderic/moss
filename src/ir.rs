use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::vec;

use strum_macros::Display;

use crate::ast::*;
use crate::lexer::Token;
use crate::symbol_table::{SymbolTable, VariableInfo};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ValueType {
    ID(u64),
    CONST(u64),
}

impl Display for ValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ID(id) => write!(f, "{}", id),
            Self::CONST(n) => write!(f, "{}", n),
        }
    }
}

struct IdStore {
    next_id: u64,
}

impl IdStore {
    pub fn new() -> Self {
        IdStore {
            next_id: 0,
        }
    }

   pub fn next(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, strum_macros::Display)]
enum Op {
    CONST,
    COPY,
    NEG,
    NOT,
    ADD,
}

// Representation of SSA.
// Arguments will references to Values stored in the basic block
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value {
    id: u64,
    op: Op,
    arg1: Option<ValueType>,
    arg2: Option<ValueType>,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut s = format!("v{} = {}", self.id, self.op);
        
        if let Some(arg1) = self.arg1.clone() {
            match arg1 {
                ValueType::ID(id) => s = format!("{} v{}", s, id), 
                ValueType::CONST(c) => s = format!("{} {}", s, c),
            }
        }
        if let Some(arg2) = self.arg2.clone() {
            match arg2 {
                ValueType::ID(id) => s = format!("{} v{}", s, id), 
                ValueType::CONST(c) => s = format!("{} {}", s, c),
            }
        }
        write!(f, "{}", s)
    }
}

pub struct ExprInfo {
    // id of the final value of the expression
    pub id: u64,
    // ssa to calculate the expr
    pub code: Vec<Value>,
    // Type of the expr for semantic analysis
    pub var_type: VariableType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fn {
    name: String,
    code: Vec<Value>,
}

impl Fn {
    pub fn new(name: &str) -> Self {
        Fn {
            name: String::from(name),
            code: Vec::new(),
        }
    }

    pub fn add_value(&mut self, val: Value) {
        self.code.push(val);
    }

    pub fn add_code(&mut self, code: &mut Vec<Value>) {
        self.code.append(code);
    }
}

impl Display for Fn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:\n", self.name).unwrap();
        self.code
            .iter()
            .for_each(|val| write!(f, "\t{}\n", val).expect("bb not found"));
        Ok(())
    }
}

pub struct IR {
    fns: Vec<Fn>,
    scopes: Vec<SymbolTable>,
    id_store: IdStore,
}

impl Display for IR {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.fns
            .iter()
            .for_each(|func| write!(f, "{}", func).expect("fn not found"));
        Ok(())
    }
}

impl IR {
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            scopes: Vec::new(),
            id_store: IdStore::new(),
        }
    }

    fn current_scope(&mut self) -> &mut SymbolTable {
        self.scopes.last_mut().unwrap()
    }

    pub fn gen_ssa(&mut self, node: &Node) -> Vec<Fn> {
        // Create the global symbol table
        self.scopes.push(SymbolTable::new());
        if let Node::Program(decls) = node {
            decls.iter().for_each(|decl| {
                let result = self.gen_decl(decl);
                if let Some(fnir) = result {
                    self.fns.push(fnir);
                }
            });
        } else {
            panic!("expecting program to generate IR!");
        }
        self.fns.clone()
    }

    fn gen_decl(&mut self, decl: &Declaration) -> Option<Fn> {
        match decl {
            Declaration::FunctionDeclaration(function) => {
                // Functions create a new block and therefore get a new symbol table
                self.scopes.push(SymbolTable::new());
                let mut fnir = Fn::new(&function.name);
                function
                    .stmts
                    .iter()
                    .for_each(|stmt| fnir.add_code(&mut self.gen_stmt(&stmt)));

                Some(fnir)
            }
            _ => None,
        }
    }

    fn gen_stmt(&mut self, stmt: &Statement) -> Vec<Value> {
        match stmt {
            Statement::VariableDeclaration(var) => self.gen_var_decl(var),
            _ => panic!("cannot generate ssa for statement"),
        }
    }

    fn gen_var_decl(&mut self, var: &Variable) -> Vec<Value> {
        if let Some(expr) = var.value.clone() {
            let expr_info = self.gen_expr(&expr);
            self.current_scope().define_variable(
                &var.name,
                VariableInfo {
                    id: expr_info.id,
                    var_type: var.var_type.clone(),
                },
            );
            expr_info.code
        } else {
            panic!("variable has no value")
        }
    }

    fn gen_expr(&mut self, expr: &Expression) -> ExprInfo {
        let mut code = Vec::new();
        let var_type: VariableType;
        match expr {
            Expression::Identifier(ident) => {
                let var_info = self.current_scope().get_variable(ident).unwrap();
                ExprInfo { id: var_info.id, code, var_type: var_info.var_type.clone() }
            }
            Expression::Number(n) => {
                let val = Value {
                    id: self.id_store.next(),
                    op: Op::CONST,
                    arg1: Some(ValueType::CONST(*n)),
                    arg2: None,
                };
                code.push(val);
                var_type = VariableType {
                    ident: String::from("u64"),
                    pointer: false,
                    array: false,
                    array_size: 0,
                };
                ExprInfo { id: code.last().unwrap().id, code, var_type }
            }
            Expression::Prefix(prefix) => self.gen_prefix_expr(prefix), 
            _ => panic!("not a valid expression for gen"),
        }
    }

    fn gen_prefix_expr(&mut self, prefix: &PrefixExpression) -> ExprInfo {
        let mut code = Vec::new();
        match prefix.op {
            Token::DASH => {
                let mut expr_info = self.gen_expr(&prefix.operand);
                code.append(&mut expr_info.code);
                let val = Value {
                    id: self.id_store.next(),
                    op: Op::NEG,
                    arg1: Some(ValueType::ID(expr_info.id)),
                    arg2: None
                };
                code.push(val);
                ExprInfo { id: code.last().unwrap().id,  code, var_type: expr_info.var_type }
            }
            _ => panic!("not a valid prefix operator!")
        }
    }
}

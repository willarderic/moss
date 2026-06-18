use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::vec;

use strum_macros::Display;

use crate::ast::*;
use crate::symbol_table::SymbolTable;

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

struct IdStore<'a> {
    next_fn_id: u64,
    next_bb_id: u64,
    next_val_id: u64,
    fns: HashMap<u64, &'a Fn>,
    bbs: HashMap<u64, &'a BasicBlock>,
    vals: HashMap<u64, &'a Value>,
}

impl<'a> IdStore<'a> {
    pub fn new() -> Self {
        IdStore {
            next_fn_id: 0,
            next_bb_id: 0,
            next_val_id: 0,
            fns: HashMap::new(),
            bbs: HashMap::new(),
            vals: HashMap::new(),
        }
    }

    pub fn get_bb_id(&mut self) -> u64 {
        self.next_bb_id += 1;
        self.next_bb_id
    }

    pub fn get_fn_id(&mut self) -> u64 {
        self.next_fn_id += 1;
        self.next_fn_id
    }

    pub fn get_val_id(&mut self) -> u64 {
        self.next_val_id += 1;
        self.next_val_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq, strum_macros::Display)]
enum Op {
    CONST,
    ASSIGN,
    ADD,
    LOAD,
    STORE,
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

impl Value {
    pub fn new(
        id_store: &mut IdStore,
        op: Op,
        arg1: Option<ValueType>,
        arg2: Option<ValueType>,
    ) -> Self {
        Value {
            id: id_store.get_val_id(),
            op,
            arg1,
            arg2,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.op {
            Op::CONST => {
                let mut s = format!("v{} = {}", self.id, self.op);
                if let Some(arg1) = self.arg1.clone() {
                    s = format!("{} {}", s, arg1); 
                }
                if let Some(arg2) = self.arg2.clone() {
                    s = format!("{}, {}", s, arg2); 
                }
                write!(f, "{}", s)
            },
            _ => panic!("operation not implemented for display"),
        }
    }
}

// Basic block is a contiguous sequence of SSA instructions
// the basic block will own the Value structs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BasicBlock {
    id: u64,
    vals: Vec<Value>,
}

impl BasicBlock {
    pub fn new(id_store: &mut IdStore) -> Self {
        BasicBlock {
            id: id_store.get_bb_id(),
            vals: Vec::new(),
        }
    }

    pub fn add_val(&mut self, val: Value) {
        self.vals.push(val);
    }

    pub fn add_vals(&mut self, vals: &mut Vec<Value>) {
        self.vals.append(vals)
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "b{}:\n", self.id).unwrap();
        self.vals
            .iter()
            .for_each(|val| write!(f, "\t\t{}\n", val).expect("value not found"));
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fn {
    id: u64,
    basic_blocks: Vec<BasicBlock>,
}

impl Fn {
    pub fn new(id_store: &mut IdStore) -> Self {
        Fn {
            id: id_store.get_fn_id(),
            basic_blocks: Vec::new(),
        }
    }

    pub fn add_basic_block(&mut self, basic_block: BasicBlock) {
        self.basic_blocks.push(basic_block);
    }

    pub fn add_basic_blocks(&mut self, basic_blocks: &mut Vec<BasicBlock>) {
        self.basic_blocks.append(basic_blocks);
    }
}

impl Display for Fn {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "f{}:\n", self.id).unwrap();
        self.basic_blocks
            .iter()
            .for_each(|bb| write!(f, "\t{}\n", bb).expect("bb not found"));
        Ok(())
    }
}

pub struct IR<'a> {
    fns: Vec<Fn>,
    scopes: Vec<SymbolTable>,
    id_store: IdStore<'a>,
}

impl<'a> Display for IR<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.fns
            .iter()
            .for_each(|func| write!(f, "{}", func).expect("fn not found"));
        Ok(())
    }
}

impl<'a> IR<'a> {
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
                let mut fnir = Fn::new(&mut self.id_store);
                function
                    .stmts
                    .iter()
                    .for_each(|stmt| fnir.add_basic_blocks(&mut self.gen_stmt(&stmt)));

                Some(fnir)
            }
            _ => None,
        }
    }

    fn gen_stmt(&mut self, stmt: &Statement) -> Vec<BasicBlock> {
        // Functions create a new block and therefore get a new symbol table
        self.scopes.push(SymbolTable::new());
        match stmt {
            Statement::VariableDeclaration(var) => vec![self.gen_var_decl(var)],
            _ => panic!("cannot generate ssa for statement"),
        }
    }

    fn gen_var_decl(&mut self, var: &Variable) -> BasicBlock {
        let mut bb = BasicBlock::new(&mut self.id_store);
        self.current_scope().define_variable(&var.name, &var.var_type);
        let mut vals = if let Some(expr) = var.value.clone() {
            self.gen_expr(&expr) 
        } else {
            panic!("variable has no value")
        };
        bb.add_vals(&mut vals);
        bb
    }

    fn gen_expr(&mut self, expr: &Expression) -> Vec<Value> {
        let mut vals = Vec::new();
        match expr {
            Expression::Number(n) => {
                let val = Value::new(&mut self.id_store, Op::CONST, Some(ValueType::CONST(*n)), None); 
                vals.push(val);
            },
            _ => panic!("not a valid expression for gen")
        };

        vals
    }
}

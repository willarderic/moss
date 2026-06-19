use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::str::FromStr;
use std::string::ToString;
use std::vec;

use strum::IntoEnumIterator;
use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::EnumString;

use crate::ast::*;
use crate::lexer::Token;
use crate::symbol_table::{SymbolTable, TypeInfo, VariableInfo};

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
)]
enum BuiltInTypes {
    // integer types
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    char,
    // Boolean
    bool,
}

fn builtin_type_size(builtin_type: BuiltInTypes) -> u64 {
    match builtin_type {
        BuiltInTypes::u8 | BuiltInTypes::i8 | BuiltInTypes::char | BuiltInTypes::bool => 1,
        BuiltInTypes::u16 | BuiltInTypes::i16 => 2,
        BuiltInTypes::u32 | BuiltInTypes::i32 => 4,
        BuiltInTypes::u64 | BuiltInTypes::i64 => 8,
    }
}

fn is_numeric_type(type_name: &str) -> bool {
    let builtin_type = match BuiltInTypes::from_str(type_name) {
        Ok(t) => t,
        Err(_) => return false,
    };
    match builtin_type {
        BuiltInTypes::u8
        | BuiltInTypes::i8
        | BuiltInTypes::char
        | BuiltInTypes::u16
        | BuiltInTypes::i16
        | BuiltInTypes::u32
        | BuiltInTypes::i32
        | BuiltInTypes::u64
        | BuiltInTypes::i64 => true,
        _ => false,
    }
}

fn is_bool_type(type_name: &str) -> bool {
    let builtin_type = match BuiltInTypes::from_str(type_name) {
        Ok(t) => t,
        Err(_) => return false,
    };
    match builtin_type {
        BuiltInTypes::bool => true,
        _ => false,
    }
}

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
        IdStore { next_id: 0 }
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
    // arithmetic ops
    #[strum(to_string = "-")]
    NEG,
    #[strum(to_string = "+")]
    ADD,
    #[strum(to_string = "*")]
    MULT,
    #[strum(to_string = "-")]
    SUB,
    #[strum(to_string = "/")]
    DIV,
    // logic ops
    #[strum(to_string = "~")]
    NOT,
    #[strum(to_string = "&")]
    AND,
    #[strum(to_string = "|")]
    OR,
    #[strum(to_string = "^")]
    XOR,
    // relops
    #[strum(to_string = "<")]
    LT,
    #[strum(to_string = ">")]
    GT,
    #[strum(to_string = "<=")]
    LEQ,
    #[strum(to_string = ">=")]
    GEQ,
    #[strum(to_string = "==")]
    EQ,
    #[strum(to_string = "!=")]
    NEQ,
    RET_VAL,
    RET
}

fn valid_binop_and_type(op: &Op, type_name: &str) -> bool {
    match op {
        // arith ops most restrictive operations. Can only be performed on numbers
        Op::ADD | Op::MULT | Op::SUB | Op::DIV | Op::LT | Op::GT | Op::LEQ | Op::GEQ => {
            is_numeric_type(type_name)
        }
        // logic ops less restictive, but can only be done on numbers and bools
        Op::NOT | Op::AND | Op::OR | Op::XOR => {
            is_numeric_type(type_name) || is_bool_type(type_name)
        }
        // Other things such as EQ and NEQ can be performed on any types.
        _ => true,
    }
}

// Representation of SSA. Essential a triple/quad data structure
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
                ValueType::ID(id) => s = format!("{}, v{}", s, id),
                ValueType::CONST(c) => s = format!("{}, {}", s, c),
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

    fn add_builtins(&mut self) {
        for builtin_type in BuiltInTypes::iter() {
            self.global_scope().define_type(
                &format!("{}", builtin_type),
                TypeInfo {
                    size: builtin_type_size(builtin_type),
                },
            );
        }
    }

    fn global_scope(&mut self) -> &mut SymbolTable {
        self.scopes.first_mut().unwrap()
    }

    fn current_scope(&mut self) -> &mut SymbolTable {
        self.scopes.last_mut().unwrap()
    }

    // TODO: will types only exist in the global scope? Is this function required?
    fn get_type_all_scopes(&self, ident: &str) -> Option<&TypeInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(type_info) = scope.get_type(ident) {
                return Some(type_info);
            }
        }
        None
    }

    fn get_var_all_scopes(&self, ident: &str) -> Option<&VariableInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get_variable(ident) {
                return Some(var_info);
            }
        }
        None
    }
    
    pub fn update_variable(&mut self, ident: &str, var_info: VariableInfo) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(_) = scope.get_variable(ident) {
                scope.define_variable(ident, var_info);
                break;
            }
        }
    }


    pub fn gen_ssa(&mut self, node: &Node) -> Vec<Fn> {
        // Create the global symbol table
        self.scopes.push(SymbolTable::new());
        self.add_builtins();
        println!("Global {}", self.scopes.last().unwrap());
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
                // end of the functions scope
                self.scopes.pop();
                Some(fnir)
            }
            _ => None,
        }
    }

    fn gen_stmt(&mut self, stmt: &Statement) -> Vec<Value> {
        match stmt {
            Statement::VariableDeclaration(var) => self.gen_var_decl(var),
            Statement::ExpressionStatement(expr) => self.gen_expr(expr).code,
            Statement::ReturnStatement(expr) => self.gen_ret_stmt(expr),
            _ => panic!("cannot generate ssa for statement"),
        }
    }

    fn gen_var_decl(&mut self, var: &Variable) -> Vec<Value> {
        let expr_info: ExprInfo;
        if let Some(expr) = var.value.clone() {
            // if we are declaring and assigning the variable to a value
            // calculate the value of the expression
            expr_info = self.gen_expr(&expr);
        } else {
            // if this is just a declaration, assign the value of the
            // variable to 0.
            let val = Value {
                id: self.id_store.next(),
                op: Op::CONST,
                arg1: Some(ValueType::CONST(0)),
                arg2: None,
            };
            expr_info = ExprInfo {
                id: val.id,
                code: vec![val],
                var_type: var.var_type.clone(),
            };
        }
        // Check if the type we are declaring the variable as exists
        self.get_type_all_scopes(&var.var_type.ident)
            .expect(&format!(
                "cannot declare variable as {}: type does not exit",
                var.var_type.ident
            ));
        if let Some(_) = self.current_scope().get_variable(&var.name) {
            panic!("redefinition of variable!!");
        }
        self.current_scope().define_variable(
            &var.name,
            VariableInfo {
                id: expr_info.id,
                var_type: var.var_type.clone(),
            },
        );
        expr_info.code
    }

    fn gen_ret_stmt(&mut self, expr: &Expression) -> Vec<Value> {
        let mut code = Vec::new();
        let mut expr_info = self.gen_expr(expr);
        code.append(&mut expr_info.code);
        code.push(Value {
            id: self.id_store.next(),
            op: Op::RET_VAL,
            arg1: Some(ValueType::ID(expr_info.id)),
            arg2: None,
        });
        code.push(Value {
            id: self.id_store.next(),
            op: Op::RET,
            arg1: None,
            arg2: None,
        });

        code
    }

    fn gen_expr(&mut self, expr: &Expression) -> ExprInfo {
        let mut code = Vec::new();
        let var_type: VariableType;
        match expr {
            Expression::Identifier(ident) => {
                let var_info = self.get_var_all_scopes(ident).unwrap();
                ExprInfo {
                    id: var_info.id,
                    code,
                    var_type: var_info.var_type.clone(),
                }
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
                ExprInfo {
                    id: code.last().unwrap().id,
                    code,
                    var_type,
                }
            }
            Expression::Prefix(prefix) => self.gen_prefix_expr(prefix),
            Expression::Infix(infix) => self.gen_infix_expr(infix),
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
                    arg2: None,
                };
                code.push(val);
                ExprInfo {
                    id: code.last().unwrap().id,
                    code,
                    var_type: expr_info.var_type,
                }
            }
            _ => panic!("not a valid prefix operator!"),
        }
    }

    fn gen_infix_expr(&mut self, infix: &InfixExpression) -> ExprInfo {
        let mut code = Vec::new();
        let mut expr_info_left = self.gen_expr(&infix.left);
        let mut expr_info_right = self.gen_expr(&infix.right);
        if expr_info_left.var_type.ident != expr_info_right.var_type.ident {
            panic!("type mismatch for {}", infix.op);
        }
        let op = match infix.op {
            Token::ASSIGN => {
                // assignment is handled different than all the other cases
                // check that an ident is on the left side
                // check that the types on both sides match
                let expr = *infix.left.clone();
                if let Expression::Identifier(ident) = expr {
                    // check that the variable is defined
                    let mut var_info = self.get_var_all_scopes(&ident)
                        .expect(&format!("{} is not defined", ident)).clone();
                    code.append(&mut expr_info_right.code);

                    // update the variable in the symbol table
                    var_info.id = code.last().unwrap().id; 
                    self.update_variable(&ident, var_info);
                    return ExprInfo {
                        id: code.last().unwrap().id,
                        code,
                        var_type: expr_info_left.var_type,
                    };
                } else {
                    panic!("left side of assignment is not a variable");
                }
            }
            Token::PLUS => Op::ADD,
            Token::ASTERISK => Op::MULT,
            Token::DASH => Op::SUB,
            Token::SLASH => Op::DIV,
            Token::LT => Op::LT,
            Token::GT => Op::GT,
            Token::LEQ => Op::LEQ,
            Token::GEQ => Op::GEQ,
            Token::EQ => Op::EQ,
            Token::NEQ => Op::NEQ,
            _ => panic!("not a valid binary operator"),
        };
        // ensure the types of each side of the binop are compatible with the op
        if !valid_binop_and_type(&op, &expr_info_left.var_type.ident) {
            // let err = format!("left expr type {} not compatible with {}", expr_info_left.var_type.ident, op.to_string());
            panic!("op and type mismatch for left expr");
        }
        if !valid_binop_and_type(&op, &expr_info_right.var_type.ident) {
            // let err = format!("left expr type {} not compatible with {}", expr_info_left.var_type.ident, op.to_string());
            panic!("op and type mismatch for right expr");
        }
        // TODO: for numeric types, resolve the size and signedness of the type to use
        // instead of just taking the left ones type for the type of the result
        let var_type = match &op {
            Op::ADD | Op::MULT | Op::SUB | Op::DIV => expr_info_left.var_type,
            Op::LT | Op::GT | Op::LEQ | Op::GEQ | Op::EQ | Op::NEQ => VariableType {
                ident: String::from("bool"),
                pointer: false,
                array: false,
                array_size: 0,
            },
            _ => panic!("not a valid binop"),
        };

        code.append(&mut expr_info_left.code);
        code.append(&mut expr_info_right.code);

        let val = Value {
            id: self.id_store.next(),
            op,
            arg1: Some(ValueType::ID(expr_info_left.id)),
            arg2: Some(ValueType::ID(expr_info_right.id)),
        };
        code.push(val);
        ExprInfo {
            id: code.last().unwrap().id,
            code,
            var_type,
        }
    }
}

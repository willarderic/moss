use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result};
use std::str::FromStr;
use std::vec;

use strum::IntoEnumIterator;

use crate::ast::*;
use crate::lexer::Token;
use crate::symbol_table::{SymbolTable, TypeInfo, VariableInfo, ValueId, BlockId};

/*
 * SSA generation is going to be implemented based on https://dl.acm.org/doi/pdf/10.1145/197320.197331
 */

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
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Char,
    // Boolean
    Bool,
}

fn builtin_type_size(builtin_type: BuiltInTypes) -> u64 {
    match builtin_type {
        BuiltInTypes::U8 | BuiltInTypes::I8 | BuiltInTypes::Char | BuiltInTypes::Bool => 1,
        BuiltInTypes::U16 | BuiltInTypes::I16 => 2,
        BuiltInTypes::U32 | BuiltInTypes::I32 => 4,
        BuiltInTypes::U64 | BuiltInTypes::I64 => 8,
    }
}

fn is_numeric_type(type_name: &str) -> bool {
    let builtin_type = match BuiltInTypes::from_str(type_name) {
        Ok(t) => t,
        Err(_) => return false,
    };
    match builtin_type {
        BuiltInTypes::U8
        | BuiltInTypes::I8
        | BuiltInTypes::Char
        | BuiltInTypes::U16
        | BuiltInTypes::I16
        | BuiltInTypes::U32
        | BuiltInTypes::I32
        | BuiltInTypes::U64
        | BuiltInTypes::I64 => true,
        _ => false,
    }
}

fn is_bool_type(type_name: &str) -> bool {
    let builtin_type = match BuiltInTypes::from_str(type_name) {
        Ok(t) => t,
        Err(_) => return false,
    };
    match builtin_type {
        BuiltInTypes::Bool => true,
        _ => false,
    }
}

#[derive(Hash, Clone, Debug, Eq, PartialEq)]
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

#[derive(Hash, Clone, Debug, Eq, PartialEq, strum_macros::Display)]
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
    GOTO,
    IF,
    IF_FALSE,
    RET_VAL,
    RET,
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
#[derive(Hash, Clone, Debug, Eq, PartialEq)]
pub struct Value {
    id: ValueId,
    op: Op,
    args: Vec<ValueType>,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut s = format!("v{} = {}", self.id, self.op);
        for arg in self.args.iter() {
            match arg {
                ValueType::ID(id) => s = format!("{} v{}", s, id),
                ValueType::CONST(c) => s = format!("{} {}", s, c),
            }
        }
        write!(f, "{}", s)
    }
}

pub struct ExprInfo {
    // id of the final value of the expression
    pub end: ValueId,
    // The id of the start of the expression
    pub start: ValueId,
    // Type of the expr for semantic analysis
    pub var_type: VariableType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum BlockType {
    ENTRY,
    PLAIN,
    DEAD,
    EXIT,
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ENTRY => write!(f, "ENTRY"),
            Self::PLAIN => write!(f, "PLAIN"),
            Self::DEAD => write!(f, "DEAD"),
            Self::EXIT => write!(f, "EXIT"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicBlock {
    pub id: u64,
    pub preds: Vec<BlockId>,
    pub succs: Vec<BlockId>,
    pub code: Vec<Value>,
    pub block_type: BlockType,
    pub sealed: bool,
    pub filled: bool,
}

impl BasicBlock {
    fn new(id: u64) -> Self {
        BasicBlock {
            id,
            preds: Vec::new(),
            succs: Vec::new(),
            code: Vec::new(),
            block_type: BlockType::PLAIN,
            sealed: false,
            filled: false,
        }
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} [{}]:\n", self.id, self.block_type).unwrap();
        write!(f, "Predecessors: ").unwrap();
        self.preds
            .iter()
            .for_each(|bb| write!(f, "{} ", bb).unwrap());
        write!(f, "Successors: ").unwrap();
        self.succs
            .iter()
            .for_each(|bb| write!(f, "{} ", bb).unwrap());
        self.code
            .iter()
            .for_each(|val| write!(f, "\t{}\n", val).unwrap());
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    name: String,
    blocks: HashMap<BlockId, BasicBlock>,
    next_block_id: BlockId,
    next_value_id: ValueId,
}

impl Function {
    fn new(name: String) -> Self {
        Function {
            name,
            blocks: HashMap::new(),
            next_block_id: 1,
            next_value_id: 1,
        }
    }

    fn new_basic_block(&mut self) -> BlockId {
       let block_id = self.next_block_id;
       self.next_block_id += 1;
       let block = BasicBlock::new(block_id);  
       self.blocks.insert(block.id, block);
       block_id
    }

    fn new_value(&mut self, block: BlockId, op: Op, args: Vec<ValueType>) -> ValueId {
        let val_id = self.next_value_id;
        self.next_value_id += 1;
        let v = Value{
            id: val_id, 
            op: op,
            args: args
        };
        self.blocks.get_mut(&block).unwrap().code.push(v);

        val_id
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:\n", self.name).unwrap();
        Ok(())
    }
}

pub struct Compiler {
    fns: Vec<Function>,
    scopes: Vec<SymbolTable>,
    block: BlockId,
}

impl Display for Compiler {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.fns
            .iter()
            .for_each(|func| write!(f, "{}", func).expect("fn not found"));
        Ok(())
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            fns: Vec::new(),
            scopes: Vec::new(),
            block: 0,
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

    fn scope(&mut self) -> &mut SymbolTable {
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

    pub fn gen_ssa(&mut self, node: &Node) -> Vec<Function> {
        // Create the global symbol table
        self.scopes.push(SymbolTable::new());
        self.add_builtins();
        println!("Global {}", self.scopes.last().unwrap());
        if let Node::Program(decls) = node {
            decls.iter().for_each(|decl| {
                match decl {
                    Declaration::Function(fn_decl) => {
                        let mut function = Function::new(fn_decl.name.clone());
                        self.block = function.new_basic_block();
                        self.gen_block_stmt(&mut function, &fn_decl.block);
                        self.fns.push(function);
                    },
                    Declaration::Variable(var_decl) => (),
                }
                
            });
        } else {
            panic!("expecting program to generate IR!");
        }
        self.fns.clone()
    }

    fn gen_block_stmt_no_scope(&mut self, f: &mut Function, block: &BlockStmt) {
        block
            .stmts
            .iter()
            .for_each(|stmt| self.gen_stmt(f, &stmt));
    }

    fn gen_block_stmt(&mut self, f: &mut Function, block: &BlockStmt) {
        self.scopes.push(SymbolTable::new());
        let code = self.gen_block_stmt_no_scope(f, block);
        self.scopes.pop();
    }

    fn gen_stmt(&mut self, f: &mut Function, stmt: &Statement) {
        match stmt {
            Statement::VariableDeclaration(var) => self.gen_var_decl(f, var),
            Statement::ExpressionStatement(expr) => self.gen_expr(f, expr),
            Statement::IfStatement(if_stmt) => self.gen_if_stmt(if_stmt),
            Statement::ReturnStatement(expr) => self.gen_ret_stmt(expr),
            Statement::BlockStatement(block) => self.gen_block_stmt(f, block),
            Statement::ForStatement(for_stmt) => self.gen_for_stmt(for_stmt),
            _ => panic!("cannot generate ssa for statement"),
        };
    }

    fn gen_var_decl(&mut self, f: &mut Function, var: &VariableDecl) {
        let expr_info: ExprInfo;
        if let Some(expr) = var.value.clone() {
            // if we are declaring and assigning the variable to a value
            // calculate the value of the expression
            expr_info = self.gen_expr(f, &expr);
        } else {
            // if this is just a declaration, assign the value of the
            // variable to 0.

            let val_id = f.new_value(self.block, Op::CONST, vec![ValueType::CONST(0)]); 


            // TODO: What do I want to do about VariableType?? How to denote pointer or array
            // Can be multilevel pointers? Maybe a "pointer_depth" value that says
            // how much pointer indirection it is. maybe vector of array dimensions to say 
            // what the array is.
            let var_type = VariableType {
                ident: BuiltInTypes::U64.to_string(),
                depth: 0,
                dimensions: Vec::new(),
            };

            expr_info = ExprInfo {
                end: val_id,
                start: val_id,
                var_type: var_type,
            }
        }
        // Check if the type we are declaring the variable as exists
        self.get_type_all_scopes(&var.var_type.ident)
            .expect(&format!(
                "cannot declare variable as {}: type does not exit",
                var.var_type.ident
            ));

        if let Some(_) = self.scope().get_variable(&var.name) {
            panic!("redefinition of variable!!");
        }

        self.scope().define_variable(
            &var.name,
            VariableInfo {
                // The value of the variable is the last value in the expression
                id: expr_info.end,
                var_type: var.var_type.clone(),
            },
        );
    }

    fn gen_if_stmt(&mut self, if_stmt: &If) {
        let mut code = Vec::new();
        let mut expr_info = self.gen_expr(&if_stmt.cond);

        if expr_info.var_type.ident != "bool" {
            panic!("if statement condition does not evaluate to boolean");
        }
        // conditional jump to consequent code, otherwise fallthrough
        // consequent code has not be created yet, so fill in arg2 later
        let mut if_val = Value {
            id: self.id_store.get_val(),
            op: Op::IF,
            arg1: Some(ValueType::ID(expr_info.code.last().unwrap().id)),
            arg2: None,
        };
        // a jmp statement that will jump after the consequent code
        // consequent code has not been created yet, fill in the subsequent id later
        let mut goto_end = Value {
            id: self.id_store.get_val(),
            op: Op::GOTO,
            arg1: None,
            arg2: None,
        };
        let mut alternate_code = Vec::new();
        if let Some(alternate) = &if_stmt.alternate {
            alternate_code.append(&mut self.gen_stmt(&alternate));
            // id should be updated so that if we have if else statements
            // then the goto from the nested if statement will jump to
            // this goto instruction, chaining until it is after the
            // entire if statement. This happens because goto_end jumps to
            // the id after the generated if statement. Updating here makes this
            // goto the id after the nested if statement
            goto_end.id = self.id_store.get_val();
        }

        let mut consequent_code = self.gen_stmt(&if_stmt.consequent);

        if_val.arg2 = Some(ValueType::ID(consequent_code.first().unwrap().id));
        // NOTE: the jump target we are putting here does not exist yet.
        // maybe a round of dead code elimination will be necessary to ensure
        // the output compiled code is correct.
        goto_end.arg1 = Some(ValueType::ID(self.id_store.next_val()));

        code.append(&mut expr_info.code);
        code.push(if_val);
        code.append(&mut alternate_code);
        code.push(goto_end);
        code.append(&mut consequent_code);
    }

    fn gen_ret_stmt(&mut self, expr: &Expression) -> Vec<Value> {
        let mut code = Vec::new();
        let mut expr_info = self.gen_expr(expr);
        code.append(&mut expr_info.code);
        code.push(Value {
            id: self.id_store.get_val(),
            op: Op::RET_VAL,
            arg1: Some(ValueType::ID(expr_info.id)),
            arg2: None,
        });
        code.push(Value {
            id: self.id_store.get_val(),
            op: Op::RET,
            arg1: None,
            arg2: None,
        });

        code
    }

    fn gen_for_stmt(&mut self, for_stmt: &For) -> Vec<Value> {
        let mut code = Vec::new();

        self.scopes.push(SymbolTable::new());
        // Precondition is execute once
        let mut pre_code = Vec::new();
        if let Some(node) = &for_stmt.pre {
            match *node.clone() {
                Node::Declaration(decl) => match decl {
                    Declaration::Variable(var_decl) => {
                        pre_code.append(&mut self.gen_var_decl(&var_decl))
                    }
                    _ => panic!("expected variable declaration in for"),
                },
                Node::Statement(stmt) => match stmt {
                    Statement::ExpressionStatement(expr) => {
                        pre_code.append(&mut self.gen_expr(&expr).code)
                    }
                    _ => panic!("expected expression statement in for"),
                },
                _ => panic!("expected variable declaration or expression!"),
            }
        }

        // Creating a jump to jump to the condition
        let mut jmp = Value {
            id: self.id_store.get_val(),
            op: Op::GOTO,
            arg1: None,
            arg2: None,
        };

        let mut loop_body = match *for_stmt.block.clone() {
            Statement::BlockStatement(block) => self.gen_block_stmt(&block),
            _ => panic!("expected a block statement for for loop body"),
        };

        let mut post_expr_code = Vec::new();
        if let Some(expr) = &for_stmt.post {
            // post condition is executed after loop body on every iteration
            post_expr_code.append(&mut self.gen_expr(&expr).code);
        }

        let mut cond_expr_code = self.gen_expr(&for_stmt.cond).code;
        self.scopes.pop();
        code.append(&mut pre_code);
        jmp.arg1 = Some(ValueType::ID(cond_expr_code.first().unwrap().id));
        code.push(jmp);
        let val = Value {
            id: self.id_store.get_val(),
            op: Op::IF,
            arg1: Some(ValueType::ID(cond_expr_code.last().unwrap().id)),
            arg2: Some(ValueType::ID(loop_body.first().unwrap().id)),
        };
        code.append(&mut loop_body);
        code.append(&mut post_expr_code);
        code.append(&mut cond_expr_code);
        code.push(val);

        code
    }

    fn gen_expr(&mut self, f: &mut Function, expr: &Expression) -> ExprInfo {
        let mut code = Vec::new();
        let var_type: VariableType;
        match expr {
            Expression::Identifier(ident) => {
                let var_info = self.get_var_all_scopes(ident).unwrap();
                ExprInfo {
                    end: var_info.id,
                    code,
                    var_type: var_info.var_type.clone(),
                }
            }
            Expression::Number(n) => {
                let val = Value {
                    id: self.id_store.get_val(),
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
                    end: code.last().unwrap().id,
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
                    id: self.id_store.get_val(),
                    op: Op::NEG,
                    arg1: Some(ValueType::ID(expr_info.id)),
                    arg2: None,
                };
                code.push(val);
                ExprInfo {
                    end: code.last().unwrap().id,
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
                    let mut var_info = self
                        .get_var_all_scopes(&ident)
                        .expect(&format!("{} is not defined", ident))
                        .clone();
                    code.append(&mut expr_info_right.code);

                    // update the variable in the symbol table
                    var_info.id = code.last().unwrap().id;
                    self.update_variable(&ident, var_info);
                    return ExprInfo {
                        end: code.last().unwrap().id,
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
            id: self.id_store.get_val(),
            op,
            arg1: Some(ValueType::ID(expr_info_left.id)),
            arg2: Some(ValueType::ID(expr_info_right.id)),
        };
        code.push(val);
        ExprInfo {
            end: code.last().unwrap().id,
            code,
            var_type,
        }
    }
}

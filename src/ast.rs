use std::fmt::{Display, Formatter, Result};
use std::vec;

use crate::lexer::Token;

/*
program     → decl_list EOF
decl       → fn_decl
            | type_decl ";"
            | var_decl ";" ;
decl_list  → epsilon
            | decl decl_list ;
fn_decl     → "fn" IDENT "(" ")" fn_return block ;
block       → stmt_list;

decl_list   → decl decl_list ;
var_decl    → "var" IDENT var_type "=" expr
            | "var" IDENT var_type
            | "var" IDENT "=" expr ;
short_var_decl → IDENT "=" expr ;
var_type    → epsilon
            | IDENT 
            | "[" NUMBER "]" IDENT;
            | "*" IDENT
stmt        → var_decl ";"
            | for_stmt
            | if_stmt
            | expr ";"
            | return_stmt ";" ;
stmt_list   → epsilon
            | stmt stmt_list ;
for_stmt    → "for" expr block
            | "for" expr ";" expr ";" expr block ;
if_stmt     → "if" expr block then_arm ;
then_arm    → "else" block ;
return_stmt → "return" expr_list;
expr        → literal
            | unary
            | binary
            | grouping
            | call ;
expr_list   → epsilon
            | expr "," expr_list
literal     → NUMBER | BOOL | STRING | IDENT | "true" | "false" | "null" ;
grouping    → "(" expr ")" ;
unary       → ( "-" | "~" | "!" | "*" | "&" ) expr ;
binary      → expr op expr ;
op          → "==" | "!=" | "<" | "<=" | ">" | ">="
               | "+"  | "-"  | "*" | "/" ;
call        → IDENT "(" expr_list ")" ;
*/
pub enum Node {
    Program(Vec<Declaration>),
    Declaration(Declaration),
    Statement(Statement),
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Program(decls) => {
                write!(f, "PROGRAM\n").unwrap();
                decls
                    .iter()
                    .for_each(|decl| write!(f, "{}\n", decl).unwrap());

                Ok(())
            }
            Self::Declaration(decl) => write!(f, "{}", decl),
            Self::Statement(stmt) => write!(f, "{}", stmt),
        }
    }
}

// Create separate "top" level declarations and inside block declarations

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Declaration {
    FunctionDeclaration(Function),
    VariableDeclaration(Variable),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub name: String,
    pub stmts: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariableType {
    pub ident: String,
    pub pointer: bool,
    pub array: bool,
    pub array_size: u64,
}

impl Display for VariableType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "VariableType [ident: {}, pointer: {}, array: {}, array_size: {}]", self.ident, self.pointer, self.array, self.array_size)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variable {
    pub name: String,
    pub var_type: VariableType,
    pub value: Option<Expression>,
}

impl Display for Declaration {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::FunctionDeclaration(func) => {
                write!(f, "\tFUNCTION({})\n", func.name).unwrap();
                func.stmts
                    .iter()
                    .for_each(|stmt| write!(f, "\t{}\n", stmt).unwrap());
                Ok(())
            }
            Self::VariableDeclaration(var) => {
                let s = format!("\tVAR({}", var.name);
                format!("{}, {}", s, var.var_type);
                let s = match &var.value {
                    Some(v) => format!("{}, {})", s, v),
                    None => format!("{}, None)", s),
                };
                write!(f, "{}", s)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    ForStatement(For),
    IfStatement(If),
    ReturnStatement(Expression),
    ExpressionStatement(Expression),
    VariableDeclaration(Variable),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct For {
    pub pre: Option<Box<Expression>>,
    pub cond: Expression,
    pub post: Option<Box<Expression>>,
    pub block: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct If {
    pub cond: Expression,
    pub block: Vec<Statement>,
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::ReturnStatement(expr) => write!(f, "\tRETURN({})", expr),
            Self::ExpressionStatement(expr) => write!(f, "\tEXPR({})", expr),
            Self::ForStatement(for_stmt) => {
                let mut s = String::from("\tFOR(");
                if let Some(pre) = &for_stmt.pre {
                    s = format!("{}PRE: {},", s, pre);
                };
                s = format!("{}, COND: {}", s, &for_stmt.cond);
                if let Some(post) = &for_stmt.post {
                    s = format!("{}, POST: {}", s, post);
                }
                write!(f, "{})\n", s);
                for_stmt
                    .block
                    .iter()
                    .for_each(|stmt| write!(f, "\t\t{}", stmt).unwrap());

                Ok(())
            }
            Self::VariableDeclaration(var) => {
                let mut s = format!("\tVAR({}", var.name);
                s = format!("{}, {}", s, var.var_type);
                s = match &var.value {
                    Some(v) => format!("{}, {})", s, v),
                    None => format!("{}, None)", s),
                };
                write!(f, "{}", s)
            }
            Self::IfStatement(if_stmt) => {
                write!(f, "\tIF({})\n", if_stmt.cond);
                if_stmt
                    .block
                    .iter()
                    .for_each(|stmt| write!(f, "\t\t{}", stmt).unwrap());
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefixExpression {
    pub op: Token,
    pub operand: Box<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfixExpression {
    pub op: Token,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallExpression {
    pub func: String,
    pub args: Vec<Expression>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Identifier(String),
    Number(u64),
    Bool(bool),
    Prefix(PrefixExpression),
    Infix(InfixExpression),
    Call(CallExpression),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Identifier(ident) => write!(f, "{}", ident),
            Self::Number(num) => write!(f, "{}", num),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Prefix(prefix) => write!(f, "({}, {})", prefix.op, prefix.operand),
            Self::Infix(infix) => write!(f, "({}, {}, {})", infix.op, infix.left, infix.right),
            Self::Call(call) => {
                write!(f, "CALL {}(", call.func).unwrap();
                call.args
                    .iter()
                    .for_each(|arg| write!(f, "{},", arg).unwrap());
                write!(f, ")").unwrap();

                Ok(())
            }
        }
    }
}

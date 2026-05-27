use crate::ast::{
    CallExpression, Declaration, Expression, For, Function, If, InfixExpression, Node,
    PrefixExpression, Statement, Variable,
};
use crate::lexer::Token;
use crate::symbol_table::SymbolTableEntry;
use std::collections::HashMap;

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Precedence {
    NONE = 0,
    ASSIGN = 1,
    CONDITIONAL = 2,
    SUM = 3,
    PRODUCT = 4,
    PREFIX = 5,
    POSTFIX = 6,
    CALL = 7,
}

pub struct Parser {
    tokens: Vec<Token>,
    curr_token: Token,
    peek_token: Token,
    index: usize,
    symbol_tables: Vec<HashMap<String, SymbolTableEntry>>,
}

fn get_infix_precedence(tok: &Token) -> Precedence {
    match tok {
        Token::ASSIGN => Precedence::ASSIGN,
        Token::PLUS | Token::DASH => Precedence::SUM,
        Token::ASTERISK | Token::SLASH => Precedence::PRODUCT,
        Token::LT | Token::LEQ | Token::GT | Token::GEQ | Token::EQ => Precedence::CONDITIONAL,
        Token::LPAREN => Precedence::CALL,
        _ => Precedence::NONE,
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Parser {
            tokens,
            curr_token: Token::UNKNOWN,
            peek_token: Token::UNKNOWN,
            index: 0,
            symbol_tables: Vec::new(),
        };

        parser.advance();
        parser.advance();

        parser
    }

    fn advance(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = match self.tokens.get(self.index) {
            Some(token) => token.clone(),
            None => Token::EOF,
        };
        self.index += 1;
        println!("curr: {}, peek: {}", self.curr_token, self.peek_token);
    }

    fn consume(&mut self, tok: Token) {
        if self.curr_token == tok {
            self.advance();
        } else {
            panic!("Expected {}, got {}", tok, self.curr_token);
        }
    }

    pub fn parse(&mut self) -> Node {
        let mut decls = Vec::new();
        self.symbol_tables.push(HashMap::new());
        while self.curr_token != Token::EOF {
            decls.push(self.parse_decl());
        }

        Node::Program(decls)
    }

    fn parse_decl(&mut self) -> Declaration {
        match self.curr_token {
            Token::FN => self.parse_fn_decl(),
            Token::VAR => self.parse_var_decl(),
            _ => panic!("No matching declaration"),
        }
    }

    fn parse_fn_decl(&mut self) -> Declaration {
        self.consume(Token::FN);

        let name = match &self.curr_token {
            Token::IDENTIFIER(ident) => ident.clone(),
            _ => panic!("Expected identifier, got {}", self.curr_token),
        };
        self.advance();

        self.consume(Token::LPAREN);

        let mut params = Vec::new();
        let mut first = true;
        while self.curr_token != Token::RPAREN {
            if !first {
                self.consume(Token::COMMA);
            }
            let name = match &self.curr_token {
                Token::IDENTIFIER(ident) => ident.clone(),
                _ => panic!("Expected identifier, got {}", self.curr_token),
            };
            self.advance();
            let typ = match &self.curr_token {
                Token::IDENTIFIER(ident) => ident.clone(),
                _ => panic!("Expected type name, got {}", self.curr_token),
            };
            self.advance();
            params.push(Variable {
                name,
                typ: Some(typ),
                value: None,
            });
            first = false;
        }

        self.consume(Token::RPAREN);

        let mut returns = Vec::new();
        first = true;
        while self.curr_token != Token::LBRACE {
            if !first {
                self.consume(Token::COMMA);
            }
            let typ = match &self.curr_token {
                Token::IDENTIFIER(ident) => ident.clone(),
                _ => panic!("Expect type name, got {}", self.curr_token),
            };
            returns.push(typ);

            self.advance();

            first = false;
        }

        let stmts = self.parse_block();

        Declaration::FunctionDeclaration(Function {
            name,
            params,
            returns,
            stmts,
        })
    }

    fn parse_var_decl(&mut self) -> Declaration {
        self.consume(Token::VAR);

        let name = match &self.curr_token {
            Token::IDENTIFIER(ident) => ident.clone(),
            _ => panic!("Expected identifier, got {}", self.curr_token),
        };
        self.advance();
        let mut typ: Option<String> = None;
        if let Token::IDENTIFIER(ident) = &self.curr_token {
            typ = Some(ident.clone());
            self.advance();
        }

        let var = match self.curr_token {
            Token::ASSIGN => {
                self.consume(Token::ASSIGN);
                let value = Some(self.parse_expr(Precedence::NONE));
                Declaration::VariableDeclaration(Variable { name, typ, value })
            }
            Token::SEMICOLON => {
                if let None = &typ {
                    panic!("Cannot infer type of {}", name);
                }
                Declaration::VariableDeclaration(Variable {
                    name,
                    typ,
                    value: None,
                })
            }
            _ => panic!("Expected \";\" or \"=\" but got {}", self.curr_token),
        };
        self.consume(Token::SEMICOLON);

        var
    }

    fn parse_block(&mut self) -> Vec<Statement> {
        self.consume(Token::LBRACE);
        let mut stmts: Vec<Statement> = Vec::new();
        while self.curr_token != Token::RBRACE {
            stmts.push(self.parse_statement());
        }
        self.consume(Token::RBRACE);

        stmts
    }

    fn parse_statement(&mut self) -> Statement {
        match self.curr_token {
            Token::RETURN => self.parse_return_statement(),
            Token::FOR => self.parse_for_statement(),
            Token::IF => self.parse_if_statement(),
            Token::VAR => match self.parse_var_decl() {
                Declaration::VariableDeclaration(var) => Statement::VariableDeclaration(var),
                _ => panic!("Expected variable declaration"),
            },
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.consume(Token::IF);
        let cond = self.parse_expr(Precedence::NONE);
        let block = self.parse_block();

        return Statement::IfStatement(If { cond, block });
    }

    fn parse_for_statement(&mut self) -> Statement {
        self.consume(Token::FOR);
        let expr = self.parse_expr(Precedence::NONE);

        match self.curr_token {
            Token::SEMICOLON => {
                let pre = Some(Box::new(expr));
                self.consume(Token::SEMICOLON);
                let cond = self.parse_expr(Precedence::NONE);
                self.consume(Token::SEMICOLON);
                let post = Some(Box::new(self.parse_expr(Precedence::NONE)));
                let block = self.parse_block();
                return Statement::ForStatement(For {
                    pre,
                    cond,
                    post,
                    block,
                });
            }
            Token::LBRACE => {
                let cond = expr;
                let block = self.parse_block();
                return Statement::ForStatement(For {
                    pre: None,
                    cond,
                    post: None,
                    block,
                });
            }
            _ => panic!(
                "Expected \";\" or \"LBRACE\" in for statement, got {}",
                self.curr_token
            ),
        }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.consume(Token::RETURN);
        let expr = self.parse_expr(Precedence::NONE);
        self.consume(Token::SEMICOLON);
        Statement::ReturnStatement(expr)
    }

    fn parse_expr_statement(&mut self) -> Statement {
        let expr = self.parse_expr(Precedence::NONE);
        self.consume(Token::SEMICOLON);
        Statement::ExpressionStatement(expr)
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Expression {
        let mut left = match self.curr_token {
            Token::IDENTIFIER(_) => self.parse_ident_expr(),
            Token::NUMBER(_) => self.parse_num_expr(),
            Token::DASH | Token::TILDE | Token::BANG => self.parse_prefix_expr(),
            _ => panic!("Expected prefix expression, got {}", self.curr_token),
        };
        while self.curr_token != Token::SEMICOLON
            && precedence < get_infix_precedence(&self.curr_token)
        {
            left = match self.curr_token {
                Token::ASSIGN
                | Token::PLUS
                | Token::DASH
                | Token::ASTERISK
                | Token::SLASH
                | Token::LT
                | Token::LEQ
                | Token::GT
                | Token::GEQ
                | Token::EQ
                | Token::LPAREN => self.parse_infix_expr(left),
                _ => panic!("Expected infix expression, got {}", self.curr_token),
            };
        }

        left
    }

    fn parse_ident_expr(&mut self) -> Expression {
        let ident = match &self.curr_token {
            Token::IDENTIFIER(s) => s.clone(),
            _ => panic!("Expected ident, got {}", self.curr_token),
        };

        self.advance();

        Expression::Identifier(ident)
    }

    fn parse_num_expr(&mut self) -> Expression {
        let num = match self.curr_token {
            Token::NUMBER(x) => x,
            _ => panic!("Expected number but got {}", self.curr_token),
        };
        self.advance();

        Expression::Number(num)
    }

    fn parse_prefix_expr(&mut self) -> Expression {
        let op = match self.curr_token {
            Token::DASH | Token::TILDE | Token::BANG => self.curr_token.clone(),
            _ => panic!("Expected prefix operator, got {}", self.curr_token),
        };
        self.advance();
        let operand = Box::new(self.parse_expr(Precedence::PREFIX));

        Expression::Prefix(PrefixExpression { op, operand })
    }

    fn parse_infix_expr(&mut self, left: Expression) -> Expression {
        match self.curr_token {
            Token::ASSIGN
            | Token::PLUS
            | Token::DASH
            | Token::ASTERISK
            | Token::SLASH
            | Token::LT
            | Token::LEQ
            | Token::GT
            | Token::GEQ
            | Token::EQ => {
                let op = self.curr_token.clone();
                self.advance();
                let right = self.parse_expr(get_infix_precedence(&op));

                Expression::Infix(InfixExpression {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                })
            }
            Token::LPAREN => {
                let name = match left {
                    Expression::Identifier(ident) => ident.clone(),
                    _ => panic!("Expected ident for function call, got {}", left),
                };
                self.consume(Token::LPAREN);

                let mut args = Vec::new();
                let mut first = true;
                while self.curr_token != Token::RPAREN {
                    if !first {
                        self.consume(Token::COMMA);
                    }
                    let expr = self.parse_expr(Precedence::NONE);
                    args.push(expr);

                    first = false;
                }
                self.consume(Token::RPAREN);

                Expression::Call(CallExpression { func: name, args })
            }
            _ => panic!("Expected binary expression, got {}", self.curr_token),
        }
    }
}

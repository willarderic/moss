mod ast;
mod ir;
mod lexer;
mod parser;
mod symbol_table;
mod ir;

use std::env;
use std::fs;

use crate::lexer::lex;
use crate::parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("usage: ./mossc <input program>");
    }

    let file_path = &args[1];

    let contents = fs::read_to_string(file_path).expect("failed to read file");

    println!("Contents: \n{contents}");
    let tokens = lex(contents);
    println!("Tokens:");
    tokens.iter().for_each(|tok| println!("\t{}", tok));
    let mut parser = Parser::new(tokens);
    let program = parser.parse();
    println!("{}", program);
}

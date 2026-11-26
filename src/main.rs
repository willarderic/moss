mod ast;
mod lexer;
mod parser;
mod symbol_table;
mod compiler;

use std::env;
use std::fs;

use crate::lexer::lex;
use crate::parser::Parser;
use crate::compiler::Compiler;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("usage: ./mossc <input program>");
    }

    let file_path = &args[1];

    let contents = fs::read_to_string(file_path).expect("failed to read file");

    println!("Contents: \n{contents}");
    let tokens = lex(contents);
    println!("------------ Lexer Output ------------");
    tokens.iter().for_each(|tok| println!("\t{}", tok));
    let mut parser = Parser::new(tokens);
    let program = parser.parse();
    println!("------------ Parser Output ------------");
    println!("{}", program);

   let mut Compiler = Compiler::new(); 
   let _ = compiler.gen_ssa(&program);
   println!("------------ Compiler Output ------------");
   println!("{}", compiler);

}

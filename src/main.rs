use core::panic;

use clap::Parser;

mod ast;
mod lex;
mod parser;

use ast::{Statement, VirtualMachine};

#[derive(Parser, Debug)]
#[clap(version, author = "Lukasz <luki446@gmail.com> Burchard", about)]
/// Lua interpreter
struct CliOptions {
    filename: String,
    #[arg(short, long, help = "Print AST")]
    print_ast: bool,
}

fn main() {
    let options = CliOptions::parse();

    let source_code = std::fs::read_to_string(options.filename).unwrap();

    let mut parser = parser::Parser::new(&source_code);
    let mut global_map = VirtualMachine::new();

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            panic!("Error: {}", err);
        }
    };

    if options.print_ast {
        for statement in &ast.statements {
            println!("{:#?}", statement);
        }
    } else {
        ast.execute(&mut global_map).unwrap();
    }
}

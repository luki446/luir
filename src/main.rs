use clap::Parser;

mod ast;
mod lex;
mod parser;

use ast::{GlobalMap, Statement};

#[derive(Parser, Debug)]
#[clap(version, author = "Lukasz <luki446@gmail.com> Burchard", about)]
/// Lua interpreter
struct CliOptions {
    filename: String,
}

fn main() {
    let options = CliOptions::parse();

    let source_code = std::fs::read_to_string(options.filename).unwrap();

    let mut parser = parser::Parser::new(&source_code);
    let mut global_map = GlobalMap::new();

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    ast.execute(&mut global_map).unwrap();

    println!("{:#?}", global_map);
}

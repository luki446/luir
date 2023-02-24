use clap::Parser;

mod lex;
mod ast;

#[derive(Parser, Debug)]
#[clap(version, author = "Lukasz <luki446@gmail.com> Burchard", about)]
/// Lua interpreter
struct CliOptions {
    filename: String,
}

fn main() {
    let options = CliOptions::parse();

    let source_code = std::fs::read_to_string(options.filename).unwrap();

    let mut lexer = lex::Lexer::new(&source_code);

    let tokens = lexer.tokenize().unwrap();

    println!("{:#?}", tokens);
}

mod parser;
mod scanner;

use parser::Parser;
use scanner::Scanner;

fn main() -> std::io::Result<()> {
    let mut cwd = std::env::current_dir().unwrap();

    cwd.push("config.sdl");

    let source = std::fs::read_to_string(cwd)?;
    let ref mut scanner = Scanner::new(&source);

    let parser = Parser::new(scanner);
    let tags = parser.parse();

    for tag in tags {
        println!("{:#?}", tag);
    }

    Ok(())
}

mod parser;
mod scanner;

use parser::Parser;
use scanner::Scanner;
use scanner::Token;

fn main() -> std::io::Result<()> {
    let mut cwd = std::env::current_dir().unwrap();

    cwd.push("config.sdl");

    let source = std::fs::read_to_string(cwd)?;

    /* let ref mut scanner =
    Scanner::new("author \"Potato Croissant\" age=28 5 6 7 8 9 10 year=1992;"); */
    let ref mut scanner = Scanner::new(&source);

    let parser = Parser::new(scanner);
    let tags = parser.parse();

    println!("{:?}", tags);

    Ok(())
}

mod scanner;

use scanner::Scanner;
use scanner::Token;

fn main() -> std::io::Result<()> {
    let mut cwd = std::env::current_dir().unwrap();

    cwd.push("config.sdl");

    let source = std::fs::read_to_string(cwd)?;

    println!("source length: {}", source.len());

    let mut tokens: Vec<Token> = Vec::new();
    let mut scanner = Scanner::new(&source);

    while let Some(token) = scanner.scan_token() {
        tokens.push(token);
    }

    println!("{:#?}", tokens);

    Ok(())
}

mod scanner;

use scanner::Scanner;
use scanner::Token;

fn main() {
    let source = "123.5f null false on off null";
    let mut scanner = Scanner::new(source);
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(token) = scanner.scan_token() {
        tokens.push(token);
    }

    // println!("{}", tokens.len());

    for t in tokens.iter() {
        println!("{:#?}", t);
    }
}

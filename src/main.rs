mod scanner;

use scanner::Scanner;
use scanner::Token;

fn main() {
    let source = "
        age 34
        temperature 36
        height 180
    ";
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

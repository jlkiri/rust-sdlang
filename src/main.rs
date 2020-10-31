mod scanner;

use scanner::Scanner;
use scanner::TokenType;

fn main() {
    let source = "abcdefg  // werefwef\n  hijklmnop";
    let mut scanner = Scanner::new(source);
    let mut tokens: Vec<TokenType> = Vec::new();

    while let Some(token) = scanner.scan_token() {
        tokens.push(token);
    }

    println!("{}", tokens.len());

    for t in tokens.iter() {
        println!("{:#?}", t);
    }
}

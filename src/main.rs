mod scanner;

use scanner::Scanner;
use scanner::Token;

fn main() {
    /* let source = r#"
        age 28
        name "Kirill"
        height 169
    "#; */
    let source = r#"test "string"
    test2 "anotherstring""#;
    let mut scanner = Scanner::new(source);
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(token) = scanner.scan_token() {
        tokens.push(token);
    }

    // println!("{}", tokens.len());

    println!("{:#?}", tokens);
}

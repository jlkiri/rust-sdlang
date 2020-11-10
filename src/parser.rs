use crate::scanner::*;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Literal {
    Float32(f32),
    Float64(f64),
    Integer(i32),
    String(String),
}

#[derive(Clone, Debug)]
pub enum Value {
    Literal(Literal),
    Null,
}

pub struct Parser<'a> {
    scanner: &'a mut Scanner<'a>,
    previous: Option<Token>,
    current: Option<Token>,
    panic_mode: bool,
    nodes: HashMap<String, Value>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner<'a>) -> Self {
        let previous = None;
        let current = scanner.scan_token();
        Parser {
            scanner,
            previous,
            current,
            panic_mode: false,
            nodes: HashMap::new(),
        }
    }

    fn consume(&self, token: Token) {
        match token {
            Token::Semicolon => (),
            _ => panic!(),
        }
    }

    fn match_token(&mut self, token: Token) -> bool {
        match &self.current {
            Some(t) if *t == token => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn declaration(&mut self) {
        self.node_definition();
        match self.current {
            Some(Token::Semicolon) => {
                println!("detected semicolon");
                self.advance();
                println!("current after advance: {:#?}", self.current);
            }
            _ => panic!(),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        println!("advancing...");
        let previous = self.current.take();
        println!("previous: {:#?}", previous);
        self.current = self.scanner.scan_token();
        println!("current: {:#?}", self.current);
        previous
    }

    fn node_definition(&mut self) {
        let key = self.identifier();
        let attr = self.attribute();

        println!("parsed definition");

        self.nodes.insert(key, attr);
    }

    fn assignee(&mut self) -> Value {
        // if (self.match_token(match_token))
        match self.current {
            Some(Token::Identifier(s, e)) => {
                self.advance();
                Value::Literal(Literal::String(String::from(
                    self.scanner.source_slice(s, e),
                )))
            }
            Some(_) => self.literal(),
            None => panic!(),
        }
    }

    fn attribute(&mut self) -> Value {
        let assignee = self.assignee();

        /*  match self.current {
          Some(Token::Equal) => {
            // If previous is not identifier - panic!
            self.advance();
            self.literal()
          }
          _ => (),
        } */

        assignee
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn identifier(&mut self) -> String {
        match self.current {
            Some(Token::Identifier(s, e)) => {
                self.advance();
                String::from(self.scanner.source_slice(s, e))
            }
            _ => panic!(),
        }
    }

    fn literal(&mut self) -> Value {
        match self.peek() {
            Some(Token::Float64(s, e)) => {
                // self.advance();
                Value::Literal(Literal::Float64(
                    self.scanner.source_slice(*s, e - 1).parse::<f64>().unwrap(),
                ))
            } /*
            Some(Token::Integer(s, e)) => {
            self.advance();
            Value::Literal(Literal::Integer(
            self.scanner.source_slice(s, e).parse::<i32>().unwrap(),
            ))
            }
            Some(Token::String(s, e)) => {
            self.advance();
            Value::Literal(Literal::String(String::from(
            self.scanner.source_slice(s, e),
            )))
            } */
            _ => unimplemented!(),
        }
    }

    pub fn parse(mut self) -> HashMap<String, Value> {
        while let Some(_) = self.current {
            self.declaration();
        }
        self.nodes
    }
}

use crate::scanner::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(i32),
    Float(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug)]
pub struct Tag {
    name: String,
    values: Vec<Value>,
    attributes: HashMap<String, Value>,
    children: Vec<Tag>,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Self {
            name,
            values: Vec::new(),
            children: Vec::new(),
            attributes: HashMap::new(),
        }
    }
}

#[derive(Debug)]
struct Error(&'static str, usize, usize, usize);

pub struct Parser<'a> {
    scanner: &'a mut Scanner<'a>,
    previous: Option<Token>,
    current: Option<Token>,
    panic_mode: bool,
    current_tag: Option<Tag>,
    tags: Vec<Tag>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner<'a>) -> Self {
        let previous = None;
        let current = scanner.next();
        Parser {
            scanner,
            previous,
            current,
            panic_mode: false,
            tags: vec![],
            current_tag: None,
        }
    }

    fn consume(&mut self, token: Token, msg: &'static str) -> Result<Option<Token>, &str> {
        match &self.current {
            Some(t) if *t == token => Ok(self.advance()),
            _ => Err(msg),
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

    fn identifier(&mut self) -> Result<String, Error> {
        match self.current {
            Some(Token::Identifier(s, e, _)) => {
                self.advance();
                Ok(String::from(self.scanner.source_slice(s, e)))
            }
            Some(Token::Error(msg, s, e, l)) => Err(Error(msg, s, e, l)),
            Some(ref t) => {
                let (start, end, line) = t.position();
                Err(Error("Invalid identifier.", start, end, line))
            }
            None => match self.previous {
                Some(ref p) => {
                    let (start, end, line) = p.position();
                    Err(Error("Expect identifier.", start, end, line))
                }
                _ => Err(Error("Expect identifier.", 0, 0, 0)),
            },
        }
    }

    fn tag_declaration(&mut self) -> Result<Tag, Error> {
        match self.identifier() {
            Ok(name) => {
                let tag = Tag::new(name);
                Ok(tag)
            }
            Err(e) => Err(e),
        }
    }

    fn advance(&mut self) -> Option<Token> {
        let previous = self.current.take();
        self.current = self.scanner.next();
        previous
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    pub fn parse(mut self) -> Vec<Tag> {
        while self.current.is_some() {
            match self.tag_declaration() {
                Ok(tag) => self.tags.push(tag),
                Err(Error(msg, start, end, line)) => {
                    let mut report = String::new();

                    let before_newline: Vec<_> = self
                        .scanner
                        .source_slice(end, self.scanner.source_length())
                        .split("\n")
                        .collect();
                    let rctx = before_newline.first().unwrap_or(&"");

                    report.push_str(format!("Parse error at line {}: {}\n", line, msg).as_str());
                    report.push_str("   |\n");
                    report.push_str(
                        format!(
                            "{}  | {}{}\n",
                            line,
                            self.scanner.source_slice(start, end),
                            rctx
                        )
                        .as_str(),
                    );
                    print!("{}", report);
                    println!("   |{}\n", format!("{:>w$}", "^", w = 2));
                    break;
                }
            }
        }
        self.tags
    }
}

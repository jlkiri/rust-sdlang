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
    tags: Vec<Tag>,
    current_tag: Tag
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner<'a>) -> Self {
        let previous = None;
        let current = scanner.next();
        Parser {
            scanner,
            previous,
            current,
            tags: vec![],
        }
    }

    fn current_tag(&self) -> Option<&Tag> {
        self.tags.last()
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

    fn literal(&mut self) -> Result<Value, Error> {
        unimplemented!()
    }

    fn attribute(&mut self) -> Result<(String, Value), Error> {
        let name = self.identifier()?;

        match self.current {
            Some(Token::Equal(_, _, _)) => {
                self.advance();
            }
            Some(ref t) => {
                let (start, end, line) = t.position();
                return Err(Error("Unexpected identifier.", start, end, line));
            }
            None => match self.previous {
                Some(ref p) => {
                    let (start, end, line) = p.position();
                    return Err(Error("Unexpected identifier.", start, end, line));
                }
                _ => (),
            },
        }

        let literal = self.literal();

        match literal {
            Ok(value) => Ok((name, value)),
            Err(Error(_, s, e, l)) => Err(Error("Expect literal.", s, e, l)),
        }
    }
    
    fn attribute_or_literal(&mut self) -> Result<(), Error> {
        let attr = self.attribute();
        let current_tag = self.current_tag();

        match attr {
            Ok(attr) => {
                tag.attributes.insert(attr.0, attr.1);
                Ok(tag)
            }
            Err(_) => {
                let literal = self.literal();
                match literal {
                    Ok(value) => {
                        tag.values.push(value);
                        Ok(tag)
                    }
                    Err(Error(_, s, e, l)) => {
                        return Err(Error("Expect at least 1 literal or attribute.", s, e, l))
                    }
                }
            }
        }
    }

    fn tag_declaration(&mut self) -> Result<Tag, Error> {
        let identifier = self.identifier()?;
        let mut tag = Tag::new(identifier);

        self.attribute_or_literal(&mut tag) {}

        Ok(tag)
    }

    fn advance(&mut self) -> Option<Token> {
        let previous = self.current.take();
        self.current = self.scanner.next();
        previous
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn print_error(&self, msg: &str, start: usize, end: usize, line: usize) {
        let mut report = String::new();
        let source_length = self.scanner.source_length();
        let lines: Vec<_> = self
            .scanner
            .source_slice(end, source_length)
            .split("\n")
            .collect();
        let rctx = lines.first().unwrap_or(&"");

        report.push_str(format!("Syntax error at line {}: {}\n", line, msg).as_str());
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
    }

    pub fn parse(mut self) -> Vec<Tag> {
        while self.current.is_some() {
            match self.tag_declaration() {
                Ok(tag) => self.tags.push(tag),
                Err(Error(msg, start, end, line)) => {
                    self.print_error(msg, start, end, line);
                    break;
                }
            }
        }
        self.tags
    }
}

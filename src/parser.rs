use crate::scanner::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum Value {
    String(String),
    Integer(i32),
    Float(f64),
    Boolean(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug)]
pub struct Tag {
    name: String,
    values: Vec<Value>,
    attributes: HashMap<String, Value>,
    children: Vec<Tag>,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: ", self.name)?;
        for value in self.values.iter() {
            write!(f, "{} ", value)?;
        }
        write!(f, "\n")
    }
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
        match self.current {
            Some(Token::Integer(s, e, _)) => {
                self.advance();
                let int = str::parse::<i32>(self.scanner.source_slice(s, e)).unwrap();
                Ok(Value::Integer(int))
            }
            Some(Token::String(s, e, _)) => {
                self.advance();
                let string = self.scanner.source_slice(s, e);
                Ok(Value::String(String::from(string)))
            }
            Some(Token::Float64(s, e, _)) => {
                self.advance();
                let float = str::parse::<f64>(self.scanner.source_slice(s, e)).unwrap();
                Ok(Value::Float(float))
            }
            Some(Token::True(_, _, _)) => {
                self.advance();
                Ok(Value::Boolean(true))
            }
            Some(Token::False(_, _, _)) => {
                self.advance();
                Ok(Value::Boolean(false))
            }
            Some(Token::Null(_, _, _)) => {
                self.advance();
                Ok(Value::Null)
            }
            Some(Token::Error(msg, s, e, l)) => Err(Error(msg, s, e, l)),
            None | Some(_) => match self.previous {
                Some(ref p) => {
                    let (start, end, line) = p.position();
                    Err(Error("Expect literal.", start, end, line))
                }
                _ => Err(Error("Expect literal.", 0, 0, 0)),
            },
        }
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

    fn attribute_or_literal(&mut self) -> Result<(String, Value), Error> {
        let attr = self.attribute();

        match attr {
            Ok(attr) => Ok(attr),
            Err(_) => {
                let literal = self.literal();
                match literal {
                    Ok(value) => Ok((String::from(""), value)),
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

        while let Some(ref token) = self.current {
            match token {
                Token::Semicolon(_, _, _) => {
                    self.advance();
                    return Ok(tag);
                }
                _ => match self.attribute_or_literal() {
                    Ok((name, value)) if name.is_empty() => {
                        tag.values.push(value);
                    }
                    Ok((name, value)) => {
                        tag.attributes.insert(name, value);
                    }
                    Err(e) => return Err(e),
                },
            }
        }

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

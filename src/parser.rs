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
        write!(f, "Tag: {}", self.name)?;
        write!(f, "\nValues: ")?;
        for value in self.values.iter() {
            write!(f, "{} ", value)?;
        }
        write!(f, "\nAttributes: ")?;
        for attribute in self.attributes.iter() {
            write!(f, "{}={}", attribute.0, attribute.1)?;
        }
        write!(f, "\nChildren: ")?;
        for child in self.children.iter() {
            write!(f, "{}", child)?;
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

    fn identifier(&mut self) -> Result<Option<String>, Error> {
        match self.current {
            Some(Token::Identifier(s, e, _)) => {
                self.advance();
                Ok(Some(String::from(self.scanner.source_slice(s, e))))
            }
            Some(Token::Error(msg, s, e, l)) => Err(Error(msg, s, e, l)),
            Some(_) => Ok(None),
            None => Ok(None),
        }
    }

    fn literal(&mut self) -> Result<Option<Value>, Error> {
        match self.current {
            Some(Token::Integer(s, e, _)) => {
                self.advance();
                let int = str::parse::<i32>(self.scanner.source_slice(s, e)).unwrap();
                Ok(Some(Value::Integer(int)))
            }
            Some(Token::String(s, e, _)) => {
                self.advance();
                let string = self.scanner.source_slice(s, e);
                Ok(Some(Value::String(String::from(string))))
            }
            Some(Token::Float64(s, e, _)) => {
                self.advance();
                let float = str::parse::<f64>(self.scanner.source_slice(s, e)).unwrap();
                Ok(Some(Value::Float(float)))
            }
            Some(Token::True(_, _, _)) => {
                self.advance();
                Ok(Some(Value::Boolean(true)))
            }
            Some(Token::False(_, _, _)) => {
                self.advance();
                Ok(Some(Value::Boolean(false)))
            }
            Some(Token::Null(_, _, _)) => {
                self.advance();
                Ok(Some(Value::Null))
            }
            Some(Token::Error(msg, s, e, l)) => Err(Error(msg, s, e, l)),
            None | Some(_) => Ok(None),
        }
    }

    fn attribute(&mut self) -> Result<Option<(String, Value)>, Error> {
        let name = self.identifier()?;

        match name {
            Some(n) => match self.current {
                Some(Token::Equal(s, e, l)) => {
                    self.advance();

                    let literal = self.literal()?;

                    match literal {
                        Some(value) => Ok(Some((n, value))),
                        None => Err(Error("Expect literal after '='.", s, e, l)),
                    }
                }
                Some(ref t) => {
                    let (start, end, line) = t.position();
                    return Err(Error("Expect '=' after attribute name.", start, end, line));
                }
                None => match self.previous {
                    Some(ref p) => {
                        let (start, end, line) = p.position();
                        return Err(Error("Unexpected identifier.", start + 1, end + 1, line));
                    }
                    _ => Err(Error("Unexpected identifier.", 0, 0, 1)),
                },
            },
            None => Ok(None),
        }
    }

    fn attribute_or_literal(&mut self) -> Result<Option<(Option<String>, Value)>, Error> {
        let attribute = self.attribute()?;

        match attribute {
            Some(attr) => Ok(Some((Some(attr.0), attr.1))),
            None => {
                let literal = self.literal()?;
                match literal {
                    Some(value) => Ok(Some((None, value))),
                    None => Ok(None),
                }
            }
        }
    }

    fn none_error(&self, msg: &'static str) -> Error {
        match self.previous {
            Some(ref p) => {
                let (start, end, line) = p.position();
                Error(msg, start + 1, end + 1, line)
            }
            _ => Error(msg, 0, 0, 1),
        }
    }

    fn tag_declaration(&mut self) -> Result<Tag, Error> {
        let identifier = self.identifier()?;

        match identifier {
            Some(name) => {
                let mut tag = Tag::new(name);

                loop {
                    match self.current {
                        Some(Token::Semicolon(_, _, _))
                        | Some(Token::LeftBrace(_, _, _))
                        | None => break,
                        Some(_) => {
                            let attr_or_literal = self.attribute_or_literal()?;

                            match attr_or_literal {
                                Some((Some(name), value)) => {
                                    tag.attributes.insert(name, value);
                                }
                                Some((None, value)) => {
                                    tag.values.push(value);
                                }
                                None => {
                                    return Err(
                                        self.none_error("Expect literal value or attribute.")
                                    )
                                }
                            }
                        }
                    }
                }

                match self.current {
                    Some(Token::Semicolon(_, _, _)) => {
                        self.advance();
                        Ok(tag)
                    }
                    Some(Token::LeftBrace(_, _, _)) => {
                        self.advance();
                        loop {
                            match self.current {
                                Some(Token::RightBrace(_, _, _)) => {
                                    self.advance();
                                    break;
                                }
                                Some(_) => {
                                    let child_tag = self.tag_declaration()?;
                                    tag.children.push(child_tag);
                                }
                                None => return Err(self.none_error("Expect '}' after tag body.")),
                            }
                        }

                        Ok(tag)
                    }
                    Some(_) | None => Err(self.none_error("Expect ';' or '{'.")),
                }
            }
            None => Err(self.none_error("Expect identifier.")),
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

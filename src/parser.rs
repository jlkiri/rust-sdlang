use crate::scanner::*;
use std::cmp;
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
        let mut indent = 2;
        write!(f, "Tag {} {{", self.name)?;
        write!(f, "\n{:>w$}values: ", "", w = indent)?;

        // f.debug_list().entries(&self.values).finish()?;

        for (i, value) in self.values.iter().enumerate() {
            if i == self.values.len() - 1 {
                write!(f, "{}", value)?;
            } else {
                write!(f, "{}, ", value)?;
            }
        }

        if self.attributes.len() > 0 {
            write!(f, "\n{:>w$}attributes: ", "", w = indent)?;
            for attribute in self.attributes.iter() {
                write!(f, "{}={}", attribute.0, attribute.1)?;
            }
        }

        if self.children.len() > 0 {
            write!(f, "\n{:>w$}children:\n", "", w = indent)?;
            indent *= 2;

            for child in self.children.iter() {
                write!(f, "{:>w$}{}", child, "", w = indent)?;
            }
        }

        write!(f, "\n}}\n")
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
    previous: Token,
    current: Token,
    tags: Vec<Tag>,
}

impl<'a> Parser<'a> {
    pub fn new(scanner: &'a mut Scanner<'a>) -> Self {
        let previous = Token::Eof(0, 0, 1);
        let current = scanner.next().unwrap_or(Token::Eof(0, 1, 1));
        Parser {
            scanner,
            previous,
            current,
            tags: vec![],
        }
    }

    fn identifier(&mut self) -> Result<Option<String>, Error> {
        match self.current {
            Token::Identifier(s, e, _) => {
                self.advance();
                Ok(Some(String::from(self.scanner.source_slice(s, e))))
            }
            Token::Error(msg, s, e, l) => Err(Error(msg, s, e, l)),
            _ => Ok(None),
        }
    }

    fn literal(&mut self) -> Result<Option<Value>, Error> {
        match self.current {
            Token::Integer(s, e, _) => {
                self.advance();
                let int = str::parse::<i32>(self.scanner.source_slice(s, e)).unwrap();
                Ok(Some(Value::Integer(int)))
            }
            Token::String(s, e, _) => {
                self.advance();
                let string = self.scanner.source_slice(s, e);
                Ok(Some(Value::String(String::from(string))))
            }
            Token::Float64(s, e, _) => {
                self.advance();
                let float = str::parse::<f64>(self.scanner.source_slice(s, e)).unwrap();
                Ok(Some(Value::Float(float)))
            }
            Token::True(_, _, _) => {
                self.advance();
                Ok(Some(Value::Boolean(true)))
            }
            Token::False(_, _, _) => {
                self.advance();
                Ok(Some(Value::Boolean(false)))
            }
            Token::Null(_, _, _) => {
                self.advance();
                Ok(Some(Value::Null))
            }
            Token::Error(msg, s, e, l) => Err(Error(msg, s, e, l)),
            _ => Ok(None),
        }
    }

    fn attribute(&mut self) -> Result<Option<(String, Value)>, Error> {
        let name = self.identifier()?;

        match name {
            Some(n) => match self.current {
                Token::Equal(s, e, l) => {
                    self.advance();

                    let literal = self.literal()?;

                    match literal {
                        Some(value) => Ok(Some((n, value))),
                        None => Err(Error("Expect literal after '='.", s, e, l)),
                    }
                }
                Token::Eof(s, e, l) => {
                    return Err(Error("Unexpected identifier.", s, e, l));
                }
                ref t @ _ => {
                    let (start, end, line) = t.position();
                    return Err(Error("Expect '=' after attribute name.", start, end, line));
                }
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

    fn tag_declaration(&mut self) -> Result<Tag, Error> {
        let identifier = self.identifier()?;

        match identifier {
            Some(name) => {
                let mut tag = Tag::new(name);

                loop {
                    match self.current {
                        Token::Semicolon(_, _, _) | Token::LeftBrace(_, _, _) => break,
                        Token::Eof(s, e, l) => {
                            return Err(Error("Expect literal value or attribute.", s, e, l))
                        }
                        _ => {
                            let attr_or_literal = self.attribute_or_literal()?;

                            match attr_or_literal {
                                Some((Some(name), value)) => {
                                    tag.attributes.insert(name, value);
                                }
                                Some((None, value)) => {
                                    tag.values.push(value);
                                }
                                None => {
                                    let (s, e, l) = self.current.position();
                                    return Err(Error(
                                        "Expect literal value or attribute.",
                                        s,
                                        e,
                                        l,
                                    ));
                                }
                            }
                        }
                    }
                }

                match self.current {
                    Token::Semicolon(s, e, l) => {
                        if tag.values.len() == 0 && tag.attributes.len() == 0 {
                            return Err(Error("Expect literal value or attribute.", s, e, l));
                        }

                        self.advance();
                        Ok(tag)
                    }
                    Token::LeftBrace(..) => {
                        self.advance();
                        loop {
                            match self.current {
                                Token::RightBrace(..) => {
                                    self.advance();
                                    break;
                                }
                                Token::Eof(s, e, l) => {
                                    return Err(Error("Expect '}' after tag body.", s, e, l))
                                }
                                _ => {
                                    let child_tag = self.tag_declaration()?;
                                    tag.children.push(child_tag);
                                }
                            }
                        }

                        Ok(tag)
                    }
                    Token::Eof(s, e, l) => Err(Error("Expect ';' or '{'.", s, e, l)),
                    _ => {
                        let (s, e, l) = self.current.position();
                        Err(Error("Expect ';' or '{'.", s, e, l))
                    }
                }
            }
            None => {
                let (s, e, l) = self.current.position();
                Err(Error("Expect identifier.", s, e, l))
            }
        }
    }

    fn advance(&mut self) -> Token {
        let previous = self.current;
        let span = cmp::min(0, self.scanner.source_length() - 2);
        let line = self.scanner.curr_line();
        self.current = self
            .scanner
            .next()
            .unwrap_or(Token::Eof(span, span + 1, line));
        previous
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
        loop {
            match self.current {
                Token::Eof(..) => break,
                _ => match self.tag_declaration() {
                    Ok(tag) => self.tags.push(tag),
                    Err(Error(msg, start, end, line)) => {
                        self.print_error(msg, start, end, line);
                        break;
                    }
                },
            }
        }
        self.tags
    }
}

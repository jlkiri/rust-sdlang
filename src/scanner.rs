use std::iter::Peekable;
use std::str::CharIndices;

type Index = usize;
type Char = (Index, char);

#[derive(Debug)]
pub enum Token {
    Unknown,
    Node(usize, usize),
    Number(usize, usize),
    Float64(usize, usize),
    Float32(usize, usize),
    Long(usize, usize),
}

pub struct Scanner<'a> {
    source: &'a str,
    start: Option<Char>,
    current: Option<Char>,
    scanner: Peekable<CharIndices<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut scanner = source.char_indices().peekable();
        let first_char = scanner.next();
        Scanner {
            source,
            start: first_char,
            current: first_char,
            scanner,
        }
    }
    fn advance(&mut self) -> Option<Char> {
        let current = self.current;
        self.current = self.scanner.next();
        current
    }
    fn peek(&self) -> Option<Char> {
        self.current
    }
    fn peek_next(&mut self) -> Option<&Char> {
        self.scanner.peek()
    }
    fn skip_whitespace(&mut self) {
        while let Some((_, ch)) = self.peek() {
            match ch {
                ' ' | '\r' | '\t' | '\\' => {
                    self.advance();
                }
                '/' => match self.peek_next() {
                    Some((_, ch)) => {
                        if ch == &'/' {
                            loop {
                                match self.peek() {
                                    Some((_, ch)) if ch != '\n' => {
                                        self.advance();
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }
                    None => return,
                },
                _ => return,
            }
        }
    }
    fn node(&mut self) -> Token {
        while !self.is_at_end() {
            let (_, ch) = self.current.unwrap();
            if ch.is_ascii_alphabetic() || ch.is_digit(10) {
                self.advance();
            } else {
                break;
            }
        }

        let (start, end) = self.range();

        Token::Node(start, end)
    }
    fn range(&self) -> (usize, usize) {
        let (start, _) = self.start.unwrap();
        let end = match self.current {
            Some((index, _)) => index,
            None => self.source.len() - 1,
        };

        (start, end)
    }
    fn match_char(&mut self, target: char) -> bool {
        if let Some(ch) = self.peek().map(|(_, c)| c) {
            if ch == target {
                self.advance();
                return true;
            }
        }

        false
    }
    fn is_at_end(&self) -> bool {
        match self.peek() {
            None => true,
            _ => false,
        }
    }
    fn is_digit(&self, chr: Option<Char>) -> bool {
        if let Some((_, ch)) = chr {
            return ch.is_digit(10);
        }
        false
    }
    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        let (start, end) = self.range();

        if self.match_char('L') {
            return Token::Long(start, end + 1);
        }

        match self.peek() {
            Some((_, '.')) => {
                if let Some((_, next)) = self.peek_next() {
                    if next.is_digit(10) {
                        self.advance();
                    }

                    while self.is_digit(self.peek()) {
                        self.advance();
                    }

                    let (start, end) = self.range();

                    if self.match_char('d') {
                        return Token::Float64(start, end + 1);
                    }

                    if self.match_char('f') {
                        return Token::Float32(start, end + 1);
                    }

                    if self.match_char('B') {
                        match self.peek_next().map(|(_, c)| c) {
                            Some(next) if next == &'D' => {
                                self.advance();
                                return Token::Float64(start, end + 2);
                            }
                            _ => (),
                        }
                    }
                }

                return Token::Number(start, end);
            }
            _ => Token::Number(start, end),
        }
    }
    pub fn scan_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        self.start = self.current;

        match self.advance() {
            Some((_, ch)) => {
                if ch.is_ascii_alphabetic() {
                    return Some(self.node());
                }

                if ch.is_digit(10) {
                    return Some(self.number());
                }

                Some(Token::Unknown)
            }
            None => None,
        }
    }
}

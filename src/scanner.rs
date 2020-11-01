use std::iter::Peekable;
use std::str::CharIndices;

type Index = usize;
type Char = (Index, char);

#[derive(Debug)]
pub enum Token {
    Unknown,
    True,
    False,
    String(usize, usize),
    Null,
    On,
    Off,
    Date(usize, usize),
    BinaryDecimal(usize, usize),
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
                ' ' | '\r' | '\t' | '\\' | '\n' => {
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
    fn matches_source(&self, start: usize, end: usize, len: usize, rest: &str) -> bool {
        let source_rest = &self.source[start..start + len];
        end - start == len && source_rest == rest
    }
    fn try_keyword(&self) -> Token {
        let (_, ch) = self.start.unwrap();
        let (start, end) = self.range();

        match ch {
            't' if self.matches_source(start + 1, end, 3, "rue") => Token::True,
            'f' if self.matches_source(start + 1, end, 4, "alse") => Token::False,
            'n' if self.matches_source(start + 1, end, 3, "ull") => Token::Null,
            'o' if end - start > 1 && self.matches_source(start + 1, end, 1, "n") => Token::On,
            'o' if end - start > 1 && self.matches_source(start + 1, end, 2, "ff") => Token::Off,
            _ => Token::Node(start, end),
        }
    }
    fn word(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }

        self.try_keyword()
    }
    fn range(&self) -> (usize, usize) {
        let (start, _) = self.start.unwrap();
        let end = match self.current {
            Some((index, _)) => index,
            None => self.source.len(),
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
    fn is_alpha(&self, chr: Option<Char>) -> bool {
        if let Some((_, ch)) = chr {
            return ch.is_ascii_alphabetic();
        }
        false
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
                    if (*next).is_digit(10) {
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
                                return Token::BinaryDecimal(start, end + 1);
                            }
                            _ => panic!("Unknown modifier B."),
                        }
                    }
                }

                Token::Number(start, end)
            }
            Some((_, '/')) => {
                while !self.is_whitespace(self.peek()) {
                    self.advance();
                }

                match self.peek_next().map(|(_, c)| c) {
                    Some(next) if (*next).is_digit(10) => {
                        self.advance();

                        while !self.is_whitespace(self.peek()) {
                            self.advance();
                        }

                        let (start, end) = self.range();

                        Token::Date(start, end)
                    }
                    _ => panic!("Unknown token."),
                }
            }
            _ => Token::Number(start, end),
        }
    }
    fn is_whitespace(&self, chr: Option<Char>) -> bool {
        if let Some((_, ch)) = chr {
            return ch.is_whitespace();
        }
        false
    }
    fn string(&mut self) -> Token {
        while let Some((_, ch)) = self.peek() {
            if ch == '"' {
                break;
            }

            self.advance();
        }

        // Consume '"'
        self.advance();

        let (start, end) = self.range();

        Token::String(start + 1, end - 1)
    }
    pub fn scan_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        self.start = self.current;

        match self.advance() {
            Some((_, ch)) => {
                if ch.is_ascii_alphabetic() {
                    return Some(self.word());
                }

                if ch.is_digit(10) {
                    return Some(self.number());
                }

                match ch {
                    '"' => Some(self.string()),
                    _ => {
                        println!("Unknown char: {:#?}", ch);

                        Some(Token::Unknown)
                    }
                }
            }
            None => None,
        }
    }
}

use std::iter::Peekable;
use std::str::CharIndices;

type Index = usize;
type Line = usize;
type Char = (Index, char);

#[derive(Debug, PartialEq)]
pub enum Token {
    True(usize, usize, Line),
    False(usize, usize, Line),
    Null(usize, usize, Line),
    Equal(usize, usize, Line),
    Semicolon(usize, usize, Line),
    LeftBrace(usize, usize, Line),
    RightBrace(usize, usize, Line),
    Error(&'static str, usize, usize, Line),
    String(usize, usize, Line),
    Identifier(usize, usize, Line),
    Float64(usize, usize, Line),
    Integer(usize, usize, Line),
    Eof,
}

impl Token {
    pub fn position(&self) -> (usize, usize, usize) {
        match self {
            Token::True(s, e, l)
            | Token::False(s, e, l)
            | Token::Null(s, e, l)
            | Token::Equal(s, e, l)
            | Token::Semicolon(s, e, l)
            | Token::LeftBrace(s, e, l)
            | Token::RightBrace(s, e, l)
            | Token::String(s, e, l)
            | Token::Identifier(s, e, l)
            | Token::Float64(s, e, l)
            | Token::Integer(s, e, l)
            | Token::Error(_, s, e, l) => (*s, *e, *l),
            Token::Eof => (0, 0, 0),
        }
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    line: usize,
    start: Option<Char>,
    current: Option<Char>,
    scanner: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.scan_token()
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut scanner = source.char_indices().peekable();
        let first_char = scanner.next();
        Scanner {
            source,
            start: first_char,
            current: first_char,
            line: 1,
            scanner,
        }
    }

    pub fn source_length(&self) -> usize {
        self.source.len()
    }

    fn advance(&mut self) -> Option<Char> {
        let current = self.current;
        self.current = self.scanner.next();
        current
    }

    fn peek(&self) -> Option<char> {
        self.current.map(|c| c.1)
    }

    fn peek_next(&mut self) -> Option<char> {
        self.scanner.peek().map(|c| c.1)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => match self.peek_next() {
                    Some(ch) => {
                        if ch == '/' {
                            self.advance();
                            loop {
                                match self.peek() {
                                    Some(ch) if ch != ';' && ch != '\n' => {
                                        self.advance();
                                    }
                                    _ => break,
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    _ => break,
                },
                '#' => loop {
                    match self.peek() {
                        Some(ch) if ch != ';' && ch != '\n' => {
                            self.advance();
                        }
                        _ => break,
                    }
                },
                '-' => match self.peek_next() {
                    Some(ch) => {
                        if ch == '-' {
                            loop {
                                match self.peek() {
                                    Some(ch) if ch != ';' && ch != '\n' => {
                                        self.advance();
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }
                    None => break,
                },
                _ => break,
            }
        }
    }

    fn range(&self) -> (usize, usize, usize) {
        let (start, _) = self.start.unwrap();
        let end = match self.current {
            Some((index, _)) => index,
            None => self.source.len(),
        };

        (start, end, self.line)
    }

    fn is_valid_char(&self, chr: Option<char>) -> bool {
        if let Some(ch) = chr {
            return ch.is_ascii_alphabetic()
                || ch.is_digit(10)
                || ch == '_'
                || ch == ':'
                || ch == '$'
                || ch == '-';
        }
        false
    }

    fn is_digit(&self, chr: Option<char>) -> bool {
        if let Some(ch) = chr {
            return ch.is_digit(10);
        }
        false
    }

    fn matches_source(&self, start: usize, end: usize, len: usize, rest: &str) -> bool {
        let source_rest = &self.source[start..start + len];
        end - start == len && source_rest == rest
    }

    fn try_keyword(&self) -> Token {
        let (_, ch) = self.start.unwrap();
        let (start, end, line) = self.range();

        match ch {
            't' if self.matches_source(start + 1, end, 3, "rue") => Token::True(start, end, line),
            'f' if self.matches_source(start + 1, end, 4, "alse") => Token::False(start, end, line),
            'n' if self.matches_source(start + 1, end, 3, "ull") => Token::Null(start, end, line),
            _ => Token::Identifier(start, end, line),
        }
    }

    fn identifier(&mut self) -> Token {
        while self.is_valid_char(self.peek()) {
            self.advance();
        }

        self.try_keyword()
    }

    fn make_error(&mut self, msg: &'static str) -> Token {
        let (start, end, line) = self.range();
        Token::Error(msg, start, end, line)
    }

    fn float(&mut self) -> Token {
        self.advance();

        match self.peek() {
            Some(ch) if !ch.is_digit(10) => self.make_error("'.' must be followed by digit."),
            Some(_) => {
                while self.is_digit(self.peek()) {
                    self.advance();
                }

                if let Some(ch) = self.peek() {
                    if ch == 'e' || ch == 'E' {
                        self.advance();

                        if let Some(ch) = self.peek() {
                            if ch == '+' || ch == '-' {
                                self.advance();

                                while self.is_digit(self.peek()) {
                                    self.advance();
                                }
                            }

                            while self.is_digit(self.peek()) {
                                self.advance();
                            }
                        } else {
                            return self.make_error("Illegal float.");
                        }
                    }
                }

                let (start, end, line) = self.range();

                return Token::Float64(start, end, line);
            }
            _ => self.make_error("'.' must be followed by digit."),
        }
    }

    fn number(&mut self) -> Token {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        let (start, end, line) = self.range();

        match self.peek() {
            Some('.') => self.float(),
            _ => Token::Integer(start, end, line),
        }
    }

    fn string(&mut self) -> Token {
        loop {
            match self.peek() {
                Some(ch) if ch != '"' => {
                    self.advance();
                }
                _ => break,
            }
        }

        // Consume '"'
        match self.advance() {
            None => return self.make_error("Unterminated string."),
            _ => (),
        }

        let (start, end, line) = self.range();

        Token::String(start + 1, end - 1, line)
    }

    pub fn source_slice(&self, start: usize, end: usize) -> &str {
        &self.source[start..end]
    }

    pub fn scan_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        self.start = self.current;

        match self.advance() {
            Some((_, ch)) => {
                if ch.is_ascii_alphabetic() || ch == '_' {
                    return Some(self.identifier());
                }

                if ch.is_digit(10) {
                    return Some(self.number());
                }

                let (start, end, line) = self.range();

                match ch {
                    '"' => Some(self.string()),
                    '=' => Some(Token::Equal(start, end, line)),
                    ';' => Some(Token::Semicolon(start, end, line)),
                    '{' => Some(Token::LeftBrace(start, end, line)),
                    '}' => Some(Token::RightBrace(start, end, line)),
                    _ => Some(self.make_error("Unexpected character.")),
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($source:expr, $exp:expr) => {
            let tokens = tokenize($source);
            assert_eq!($exp, tokens);
        };
    }

    fn tokenize(source: &str) -> Vec<Token> {
        let mut scanner = Scanner::new(source);
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(token) = scanner.scan_token() {
            tokens.push(token);
        }

        tokens
    }

    #[test]
    fn scan_integers() {
        test!("1", vec![Token::Integer(0, 1, 1)]);
    }

    #[test]
    fn scan_64_floats() {
        test!(
            "1.2 3.4 5.6e1 7.8e+12",
            vec![
                Token::Float64(0, 3, 1),
                Token::Float64(4, 7, 1),
                Token::Float64(8, 13, 1),
                Token::Float64(14, 21, 1),
            ]
        );
    }

    #[test]
    fn scan_64_float_error() {
        test!(
            "1.",
            vec![Token::Error("'.' must be followed by digit.", 0, 2, 1)]
        );
    }

    #[test]
    fn scan_64_float_error_2() {
        test!(
            "5.a",
            vec![
                Token::Error("'.' must be followed by digit.", 0, 2, 1),
                Token::Identifier(2, 3, 1),
            ]
        );
    }

    #[test]
    fn scan_string() {
        test!(r#""hello""#, vec![Token::String(1, 6, 1)]);
    }

    #[test]
    fn scan_identifier() {
        test!("author", vec![Token::Identifier(0, 6, 1)]);
    }

    #[test]
    fn skips_comments() {
        let source = r#"author //comment comment;
age;
"#;
        test!(
            source,
            vec![
                Token::Identifier(0, 6, 1),
                Token::Semicolon(24, 25, 1),
                Token::Identifier(26, 29, 2),
                Token::Semicolon(29, 30, 2),
            ]
        );
    }

    #[test]
    fn skips_comments_newline() {
        let source = r#"a//
age;
"#;
        test!(
            source,
            vec![
                Token::Identifier(0, 1, 1),
                Token::Identifier(4, 7, 2),
                Token::Semicolon(7, 8, 2),
            ]
        );
    }

    #[test]
    fn skips_shell_comments() {
        let source = r#"author #comment comment;
age;
"#;
        test!(
            source,
            vec![
                Token::Identifier(0, 6, 1),
                Token::Semicolon(23, 24, 1),
                Token::Identifier(25, 28, 2),
                Token::Semicolon(28, 29, 2),
            ]
        );
    }

    #[test]
    fn skips_lua_comments() {
        let source = r#"author --comment comment;
age;
"#;
        test!(
            source,
            vec![
                Token::Identifier(0, 6, 1),
                Token::Semicolon(24, 25, 1),
                Token::Identifier(26, 29, 2),
                Token::Semicolon(29, 30, 2),
            ]
        );
    }

    #[test]
    fn scan_attribute() {
        test!(
            "private=true",
            vec![
                Token::Identifier(0, 7, 1),
                Token::Equal(7, 8, 1),
                Token::True(8, 12, 1)
            ]
        );
    }

    #[test]
    fn scan_string_attribute() {
        test!(
            r#"platform="darwin""#,
            vec![
                Token::Identifier(0, 8, 1),
                Token::Equal(8, 9, 1),
                Token::String(10, 16, 1)
            ]
        );
    }

    #[test]
    fn scan_keywords() {
        test!(
            "true false null",
            vec![
                Token::True(0, 4, 1),
                Token::False(5, 10, 1),
                Token::Null(11, 15, 1)
            ]
        );
    }

    #[test]
    fn forward_slash_error() {
        test!(
            "/a",
            vec![
                Token::Error("Unexpected character.", 0, 1, 1),
                Token::Identifier(1, 2, 1),
            ]
        );
    }

    #[test]
    fn same_line_semicolon() {
        test!(
            "a ; b",
            vec![
                Token::Identifier(0, 1, 1),
                Token::Semicolon(2, 3, 1),
                Token::Identifier(4, 5, 1),
            ]
        );
    }

    #[test]
    fn semicolon() {
        test!(
            r#"author "Kirill Vasiltsov";"#,
            vec![
                Token::Identifier(0, 6, 1),
                Token::String(8, 24, 1),
                Token::Semicolon(25, 26, 1),
            ]
        );
    }

    #[test]
    fn empty() {
        test!("", vec![] as Vec<Token>);
    }
}

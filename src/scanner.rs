use std::iter::Peekable;
use std::str::CharIndices;

type Index = usize;
type Char = (Index, char);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token {
    True,
    False,
    Null,
    On,
    Off,
    Equal,
    Semicolon,
    Error(&'static str),
    String(usize, usize),
    Identifier(usize, usize),
    Float64Double(usize, usize),
    Date(usize, usize),
    Decimal128(usize, usize),
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
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\\' => match self.peek_next() {
                    Some((_, ch)) => {
                        if ch == &'\n' {
                            self.advance();
                            self.advance();
                        }
                    }
                    _ => break,
                },
                '/' => match self.peek_next() {
                    Some((_, ch)) => {
                        if ch == &'/' {
                            self.advance();
                            loop {
                                match self.peek() {
                                    Some((_, ch)) if ch != ';' && ch != '\n' => {
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
                        Some((_, ch)) if ch != ';' && ch != '\n' => {
                            self.advance();
                        }
                        _ => break,
                    }
                },
                '-' => match self.peek_next() {
                    Some((_, ch)) => {
                        if ch == &'-' {
                            loop {
                                match self.peek() {
                                    Some((_, ch)) if ch != ';' && ch != '\n' => {
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

    fn is_whitespace(&self, chr: Option<Char>) -> bool {
        if let Some((_, ch)) = chr {
            return ch.is_whitespace();
        }
        true
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
            _ => Token::Identifier(start, end),
        }
    }

    fn identifier(&mut self) -> Token {
        while self.is_alpha(self.peek()) || self.is_digit(self.peek()) {
            self.advance();
        }

        self.try_keyword()
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
                            match self.peek().map(|(_, c)| c) {
                                Some(c) if c == 'D' => {
                                    self.advance();
                                    return Token::Decimal128(start, end + 2);
                                }
                                _ => return Token::Error("Unknown number type B."),
                            }
                        }

                        return Token::Float64Double(start, end);
                    }

                    return Token::Error("'.' must be followed by digit.");
                }

                Token::Error("Number cannot end with '.'")
            }
            Some((_, '/')) | Some((_, ':')) => match self.peek_next().map(|(_, c)| c) {
                Some(next) if (*next).is_digit(10) => {
                    self.advance();

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

                            return Token::Date(start, end);
                        }
                        _ => (),
                    }

                    let (start, end) = self.range();

                    Token::Date(start, end)
                }
                _ => Token::Error("Invalid date."),
            },
            _ => Token::Number(start, end),
        }
    }

    fn string(&mut self) -> Token {
        loop {
            match self.peek() {
                Some((_, ch)) if ch != '"' => {
                    self.advance();
                }
                _ => break,
            }
        }

        // Consume '"'
        match self.advance() {
            None => return Token::Error("Unterminated string."),
            _ => (),
        }

        let (start, end) = self.range();

        Token::String(start + 1, end - 1)
    }

    pub fn source_slice(&self, start: usize, end: usize) -> &str {
        &self.source[start..end]
    }

    pub fn scan_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        self.start = self.current;

        match self.advance() {
            Some((_, ch)) => {
                if ch.is_ascii_alphabetic() {
                    return Some(self.identifier());
                }

                if ch.is_digit(10) {
                    return Some(self.number());
                }

                match ch {
                    '"' => Some(self.string()),
                    '=' => Some(Token::Equal),
                    ';' => {
                        if let Some((_, ch)) = self.peek() {
                            if ch == '\n' {
                                self.advance();
                            }
                        }
                        Some(Token::Semicolon)
                    }
                    '\n' => Some(Token::Semicolon),
                    _ => Some(Token::Error("Unexpected character.")),
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let tokens = tokenize("1");
        let expected = vec![Token::Number(0, 1)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_64_double_floats() {
        let tokens = tokenize("1.567");
        let expected = vec![Token::Float64Double(0, 5)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_64_floats() {
        let tokens = tokenize("1.567d");
        let expected = vec![Token::Float64(0, 6)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_32_floats() {
        let tokens = tokenize("1.567f");
        let expected = vec![Token::Float32(0, 6)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_long() {
        let tokens = tokenize("155L");
        let expected = vec![Token::Long(0, 4)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_128_decimal() {
        let tokens = tokenize("155.8BD");
        let expected = vec![Token::Decimal128(0, 7)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_string() {
        let tokens = tokenize(r#""hello""#);
        let expected = vec![Token::String(1, 6)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_identifier() {
        let tokens = tokenize("author");
        let expected = vec![Token::Identifier(0, 6)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn automatic_semicolon() {
        let source = r#"author  
age"#;
        let tokens = tokenize(source);
        let expected = vec![
            Token::Identifier(0, 6),
            Token::Semicolon,
            Token::Identifier(9, 12),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn backslash() {
        let source = r#"author  \
age"#;
        let tokens = tokenize(source);
        let expected = vec![Token::Identifier(0, 6), Token::Identifier(10, 13)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn backslash_no_semicolon() {
        let source = r#"a\
b"#;
        let tokens = tokenize(source);
        let expected = vec![Token::Identifier(0, 1), Token::Identifier(3, 4)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn skips_comments() {
        let source = r#"author //comment comment;
age
"#;
        let tokens = tokenize(source);
        let expected = vec![
            Token::Identifier(0, 6),
            Token::Semicolon,
            Token::Identifier(26, 29),
            Token::Semicolon,
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn skips_comments_newline() {
        let source = r#"a//
age
"#;
        let tokens = tokenize(source);
        let expected = vec![
            Token::Identifier(0, 1),
            Token::Semicolon,
            Token::Identifier(4, 7),
            Token::Semicolon,
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn skips_shell_comments() {
        let source = r#"author #comment comment;
age
"#;
        let tokens = tokenize(source);
        let expected = vec![
            Token::Identifier(0, 6),
            Token::Semicolon,
            Token::Identifier(25, 28),
            Token::Semicolon,
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn skips_lua_comments() {
        let source = r#"author --comment comment;
age
"#;
        let tokens = tokenize(source);
        let expected = vec![
            Token::Identifier(0, 6),
            Token::Semicolon,
            Token::Identifier(26, 29),
            Token::Semicolon,
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_attribute() {
        let tokens = tokenize("private=true");
        let expected = vec![Token::Identifier(0, 7), Token::Equal, Token::True];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_string_attribute() {
        let tokens = tokenize(r#"platform="darwin""#);
        let expected = vec![Token::Identifier(0, 8), Token::Equal, Token::String(10, 16)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_time() {
        let source = "13:23:34";
        let tokens = tokenize(source);
        let expected = vec![Token::Date(0, 8)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_date_1() {
        let source = "2015/12/06 12:00:00.000-UTC attr";
        let tokens = tokenize(source);
        let expected = vec![Token::Date(0, 27), Token::Identifier(28, 32)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_date_2() {
        let source = "2015/12/06 12:00:00.000 attr";
        let tokens = tokenize(source);
        let expected = vec![Token::Date(0, 23), Token::Identifier(24, 28)];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn scan_keywords() {
        let tokens = tokenize("off on true false null");
        let expected = vec![
            Token::Off,
            Token::On,
            Token::True,
            Token::False,
            Token::Null,
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn date_error() {
        let tokens = tokenize("2015/a");
        let expected = vec![
            Token::Error("Invalid date."),
            Token::Error("Unexpected character."),
            Token::Identifier(5, 6),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn forward_slash_error() {
        let tokens = tokenize("/a");
        let expected = vec![
            Token::Error("Unexpected character."),
            Token::Identifier(1, 2),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn number_error_1() {
        let tokens = tokenize("5.");
        let expected = vec![
            Token::Error("Number cannot end with '.'"),
            Token::Error("Unexpected character."),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn number_error_2() {
        let tokens = tokenize("5.a");
        let expected = vec![
            Token::Error("'.' must be followed by digit."),
            Token::Error("Unexpected character."),
            Token::Identifier(2, 3),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn number_error_3() {
        let tokens = tokenize("5.3BF");
        let expected = vec![
            Token::Error("Unknown number type B."),
            Token::Identifier(4, 5),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn no_newline_after_semicolon() {
        let tokens = tokenize(";\n");
        let expected = vec![Token::Semicolon];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn newline_as_semicolon() {
        let tokens = tokenize("a\nb");
        let expected = vec![
            Token::Identifier(0, 1),
            Token::Semicolon,
            Token::Identifier(2, 3),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn same_line_semicolon() {
        let tokens = tokenize("a ; b");
        let expected = vec![
            Token::Identifier(0, 1),
            Token::Semicolon,
            Token::Identifier(4, 5),
        ];
        assert_eq!(expected, tokens);
    }

    #[test]
    fn semicolon() {
        let tokens = tokenize(
            r#"author "Kirill Vasiltsov";
year 2020;"#,
        );
        let expected = vec![Token::Identifier(0, 1), Token::Semicolon];
        assert_eq!(expected, tokens);
    }
}

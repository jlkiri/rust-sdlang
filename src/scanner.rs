use std::iter::Peekable;
use std::str::CharIndices;

type Char = (usize, char);

#[derive(Debug)]
pub enum TokenType {
  Unknown,
  EOF,
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
        ' ' | '\r' | '\t' | '\n' => {
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
  pub fn scan_token(&mut self) -> Option<TokenType> {
    self.skip_whitespace();
    if let Some(ch) = self.advance() {
      return Some(TokenType::Unknown);
    }
    None
  }
}

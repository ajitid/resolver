use std::fmt;
use std::str;
use std::ops;

use crossterm::style::Stylize;

use crate::rdl::error;

pub const ESCAPE: char  = '\\';
pub const LBRACE: char  = '{';
pub const RBRACE: char  = '}';
pub const LPAREN: char  = '(';
pub const RPAREN: char  = ')';
pub const EQUAL: char   = '=';
pub const QUOTE: char   = '"';
pub const COMMA: char   = ',';
pub const ADD: char     = '+';
pub const SUB: char     = '-';
pub const DIV: char     = '/';
pub const MUL: char     = '*';
pub const MOD: char     = '%';
pub const AT: char      = '@';

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TType {
  Verbatim,
  Whitespace,
  Ident,
  Number,
  String,
  Operator,
  Assign,
  LParen,
  RParen,
  Symbol,
  End,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
  pub ttype: TType,
  pub ttext: String,
  pub range: ops::Range<usize>,
}

impl Token {
  pub fn new(ttype: TType, ttext: &str, range: ops::Range<usize>) -> Token {
    Token{
      ttype: ttype,
      ttext: ttext.to_string(),
      range: range,
    }
  }
  
  pub fn styled(&self) -> Option<String> {
    let ttext: &str = self.ttext.as_ref();
    match self.ttype {
      TType::Verbatim => Some(format!("{}", ttext.reset())),
      TType::Whitespace => Some(format!("{}", ttext.reset())),
      TType::Ident => Some(format!("{}", ttext.bold())),
      TType::Number => Some(format!("{}", ttext.yellow())),
      TType::String => Some(format!("{}", ttext.cyan())),
      TType::Operator => Some(format!("{}", ttext.green())),
      TType::Symbol => Some(format!("{}", ttext.blue())),
      _ => None,
    }
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<{}>", self.ttext)
  }
}

#[derive(Debug)]
pub struct Scanner<'a> {
  text: &'a str,
  data: str::Chars<'a>,
  tokens: Vec<Token>,
  peek: Option<char>,
  index: usize, // index in text, in bytes
}

impl<'a> fmt::Display for Scanner<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.text)
  }
}

impl<'a> Scanner<'a> {
  pub fn new(text: &'a str) -> Scanner<'a> {
    Scanner{
      text: text,
      data: text.chars(),
      tokens: Vec::new(),
      peek: None,
      index: 0,
    }
  }
  
  fn syntax_error(&mut self, m: &str) -> error::Error {
    error::SyntaxError::new(self.text, ops::Range{start: self.index, end: self.index}, m).into()
  }
  
  pub fn peek(&mut self) -> Option<char> {
    if self.peek == None {
      self.peek = self.read();
    }
    self.peek
  }
  
  pub fn skip(&mut self) {
    if let Some(c) = self.peek {
      self.index += c.len_utf8();
    }
    self.peek = self.read();
  }
  
  pub fn next(&mut self) -> Option<char> {
    let n = self.peek();
    self.skip();
    n
  }
  
  pub fn index(&self) -> usize {
    self.index
  }
  
  fn read(&mut self) -> Option<char> {
    self.data.next()
  }
  
  /// Determine if the next char in the stream passes the provided check.
  /// If so return true, otherwise return false. The next char is not consumed
  /// in any case.
  pub fn peek_fn(&mut self, check: impl Fn(char) -> bool) -> bool {
    if let Some(c) = self.peek() {
      check(c)
    }else{
      false
    }
  }
  
  /// Expect the provided char to be next in the stream and if so,
  /// return it, otherwise produce an error.
  pub fn assert(&mut self, e: char) -> Result<char, error::Error> {
    self.assert_fn(|c| { e == c })
  }
  
  /// Expect the next char in the stream to pass the provided check. If so
  /// consume it and return the char, otherwise do not consume it and
  /// produce an error.
  pub fn assert_fn(&mut self, check: impl Fn(char) -> bool) -> Result<char, error::Error> {
    if let Some(c) = self.expect_fn(check) {
      Ok(c)
    }else{
      Err(error::AssertionFailed::new().into())
    }
  }
  
  /// Expect the provided char to be next in the stream and if so, consume
  /// it an return true; otherwise do not consume it and return false.
  pub fn expect(&mut self, e: char) -> bool {
    match self.expect_fn(|c| { e == c }) {
      Some(_) => true,
      None    => false,
    }
  }
  
  /// Expect the next char in the stream to pass the provided check. If so
  /// consume it and return true, otherwise do not consume it and return false.
  pub fn expect_fn(&mut self, check: impl Fn(char) -> bool) -> Option<char> {
    if let Some(c) = self.peek() {
      if check(c) {
        self.skip();
        return Some(c);
      }
    }
    None
  }
  
  /// Look ahead for the next token type in the stream. Nothign is consumed.
  pub fn la(&mut self) -> Option<TType> {
    if self.tokens.len() == 0 {
      let _ = self.scan(); // ignore error, just produce none
    }
    if self.tokens.len() > 0 {
      Some(self.tokens[0].ttype)
    }else{
      None
    }
  }
  
  /// Look ahead for the next token type in the stream. Nothign is consumed.
  pub fn la_range(&mut self) -> Option<ops::Range<usize>> {
    if self.tokens.len() == 0 {
      let _ = self.scan(); // ignore error, just produce none
    }
    if self.tokens.len() > 0 {
      Some(self.tokens[0].range.clone())
    }else{
      None
    }
  }
  
  /// Step over and consume the next token that has already been scanned.
  /// This can be used to discard a token that has already been obtained
  /// via la(). If no token exists in the look-ahead buffer, this method
  /// does nothing.
  fn step(&mut self) -> Option<Token> {
    if self.tokens.len() > 0 {
      Some(self.tokens.remove(0))
    }else{
      None
    }
  }
  
  /// Discard the as many of the specified token that are in the stream
  /// up until but not including the first token that is of a different
  /// type. The number of tokens discarded is returned.
  pub fn discard(&mut self, which: TType) -> usize {
    self.discard_fn(|ttype| { which == ttype })
  }
  
  /// Discard the as many of the specified token that are in the stream
  /// up until but not including the first token that does not match the
  /// specified filter. The number of tokens discarded is returned.
  pub fn discard_fn(&mut self, check: impl Fn(TType) -> bool) -> usize {
    let mut n: usize = 0;
    loop {
      match self.la() {
        Some(next) => if check(next) {
          n += 1;
          self.step();
        }else{
          break;
        },
        None => break,
      };
    }
    n
  }
  
  /// Produce the next token in the stream or an error if no token can be
  /// produced. If there are no more tokens because the input stream has
  /// been fully consumed, the End token is returned.
  pub fn token(&mut self) -> Result<Token, error::Error> {
    if self.tokens.len() == 0 {
      self.scan()?;
    }
    if self.tokens.len() > 0 {
      Ok(self.tokens.remove(0))
    }else{
      Ok(Token::new(TType::End, "", 0..0))
    }
  }
  
  /// Look ahead for the next token type in the stream, expecting a certain
  /// type. If the expected type is found, return it, otherwise nothing.
  pub fn expect_token(&mut self, expect: TType) -> Result<Token, error::Error> {
    self.expect_token_fn(|ttype| { ttype == expect })
  }
  
  /// Look ahead for the next token type in the stream, expecting a certain
  /// type. If the expected type is found, return it, otherwise nothing.
  pub fn expect_token_fn(&mut self, check: impl Fn(TType) -> bool) -> Result<Token, error::Error> {
    let ttype = match self.la() {
      Some(ttype) => ttype,
      None => return Err(error::Error::TokenNotMatched),
    };
    if ttype == TType::End {
      Err(error::Error::EndOfInput)
    }else if check(ttype) {
      self.token()
    }else{
      Err(error::Error::TokenNotMatched)
    }
  }
  
  fn push(&mut self, tok: Token) {
    self.tokens.push(tok);
  }
  
  fn scan(&mut self) -> Result<(), error::Error> {
    if let Some(_) = self.peek() {
      match self.scan_semantic() {
        Ok(v)  => Ok(v),
        Err(_) => self.scan_verbatim(),
      }
    }else{
      Ok(()) // no tokens generated
    }
  }
  
  fn scan_semantic(&mut self) -> Result<(), error::Error> {
    if let Some(c) = self.peek() {
      if Self::is_ident_start(c) {
        return self.scan_ident();
      }else if Self::is_number_start(c) {
        return self.scan_number();
      }else if Self::is_operator(c) {
        return self.scan_operator();
      }else if Self::is_whitespace(c) {
        return self.scan_whitespace();
      }else if Self::is_symbol(c) {
        return self.scan_symbol();
      }
    }
    Err(error::Error::TokenNotMatched)
  }
  
  fn scan_verbatim(&mut self) -> Result<(), error::Error> {
    let idx = self.index;
    let mut buf = String::new();
    loop {
      if let Some(c) = self.peek() {
        if Self::is_ident_start(c) {
          break;
        }else if Self::is_number_start(c) {
          break;
        }else if Self::is_operator(c) {
          break;
        }else if Self::is_symbol(c) {
          break;
        }else if c == ESCAPE {
          buf.push_str(&self.escape()?)
        }else{
          buf.push(c);
          self.skip();
        }
      }else{
        break;
      }
    }
    self.push(Token{
      ttype: TType::Verbatim,
      ttext: buf,
      range: idx..self.index,
    });
    Ok(())
  }
  
  fn scan_ident(&mut self) -> Result<(), error::Error> {
    let idx = self.index;
    let name = self.ident()?;
    self.push(Token{
      ttype: TType::Ident,
      ttext: name,
      range: idx..self.index,
    });
    Ok(())
  }
  
  fn scan_number(&mut self) -> Result<(), error::Error> {
    let idx = self.index;
    let val = self.number()?;
    self.push(Token{
      ttype: TType::Number,
      ttext: val,
      range: idx..self.index,
    });
    Ok(())
  }
  
  fn scan_operator(&mut self) -> Result<(), error::Error> {
    let idx = self.index;
    let mut buf = String::new();
    while let Some(c) = self.peek() {
      if Self::is_operator(c) {
        buf.push(c);
      }else{
        break;
      }
      self.skip(); // consume the character
    }
    self.push(Token{
      ttype: TType::Operator,
      ttext: buf,
      range: idx..self.index,
    });
    Ok(())
  }
  
  fn scan_whitespace(&mut self) -> Result<(), error::Error> {
    let idx = self.index;
    let ws = self.whitespace()?;
    self.push(Token{
      ttype: TType::Whitespace,
      ttext: ws,
      range: idx..self.index,
    });
    Ok(())
  }
  
  fn scan_symbol(&mut self) -> Result<(), error::Error> {
    let idx = self.index;
    if let Some(c) = self.next() {
      let ttype = match c {
        LPAREN => TType::LParen,
        RPAREN => TType::RParen,
        EQUAL  => TType::Assign,
        _      => TType::Symbol,
      };
      self.push(Token{
        ttype: ttype,
        ttext: c.to_string(),
        range: idx..self.index,
      });
    }
    Ok(())
  }
  
  fn skip_ws(&mut self) -> Result<(), error::Error> {
    let _ = self.whitespace()?;
    Ok(())
  }
  
  fn whitespace(&mut self) -> Result<String, error::Error> {
    let mut buf = String::new();
    while let Some(c) = self.peek() {
      if c.is_whitespace() {
        buf.push(c);
      }else{
        break;
      }
      self.skip(); // consume the character
    }
    Ok(buf)
  }
  
  fn is_ident(c: char) -> bool {
    c.is_alphabetic() || c.is_digit(10) || c == '_'
  }
  
  fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
  }
  
  fn is_number_start(c: char) -> bool {
    c.is_digit(10)
  }
  
  fn is_whitespace(c: char) -> bool {
    c.is_whitespace()
  }
  
  fn is_operator(c: char) -> bool {
    c == ADD || c == SUB || c == MUL || c == DIV || c == MOD
  }
  
  fn is_symbol(c: char) -> bool {
    c == EQUAL || c == LPAREN || c == RPAREN
  }
  
  fn ident(&mut self) -> Result<String, error::Error> {
    let mut buf = String::new();
    buf.push(self.assert_fn(|c| { Self::is_ident_start(c) })?);
    while let Some(c) = self.peek() {
      if Self::is_ident(c) {
        buf.push(c);
      }else{
        break;
      }
      self.skip(); // consume the character
    }
    Ok(buf)
  }
  
  fn integer(&mut self) -> Result<String, error::Error> {
    let mut buf = String::new();
    buf.push(self.assert_fn(|c| { c.is_digit(10) })?);
    while let Some(c) = self.peek() {
      if c.is_digit(10) {
        buf.push(c);
      }else{
        break;
      }
      self.skip(); // consume the character
    }
    Ok(buf)
  }
  
  fn number(&mut self) -> Result<String, error::Error> {
    let mut buf = String::new();
    buf.push_str(&self.integer()?);
    if let Some(c) = self.peek() {
      if c == '.' {
        buf.push(c);
        self.skip();
        buf.push_str(&self.integer()?);
      }
    }
    Ok(buf)
  }
  
  fn string(&mut self) -> Result<String, error::Error> {
    let mut buf = String::new();
    self.assert(QUOTE)?;
    while let Some(c) = self.peek() {
      match c {
        ESCAPE => buf.push_str(&self.escape()?),
        QUOTE  => break,
        _      => {
          buf.push(c);
          self.skip();
        },
      }
    }
    self.assert(QUOTE)?;
    Ok(buf)
  }
  
  fn escape(&mut self) -> Result<String, error::Error> {
    self.assert(ESCAPE)?;
    if let Some(c) = self.next() { // consume immediately
      match c {
        'n'     => Ok("\n".to_string()),
        'r'     => Ok("\r".to_string()),
        '"'     => Ok("\"".to_string()),  // literal quote
        LBRACE  => Ok("{".to_string()),   // literal left brace
        LPAREN  => Ok("(".to_string()),   // literal left parenthesis
        AT      => Ok("@".to_string()),   // literal meta
        DIV     => Ok("/".to_string()),   // literal forward slash
        ESCAPE  => Ok("\\".to_string()),  // literal backslash
        _       => Err(self.syntax_error("Invalid escape sequence")),
      }
    }else{
      Err(error::Error::EndOfInput)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn peek_next() {
    let s = "Foo bar".to_string();
    let mut t = Scanner::new(&s);
    assert_eq!(Some('F'), t.peek());
    assert_eq!(Some('F'), t.peek());
    assert_eq!(Some('F'), t.peek());
    assert_eq!(Some('F'), t.next());
    assert_eq!(Some('o'), t.peek());
    assert_eq!(Some('o'), t.next());
    assert_eq!(Some('o'), t.peek());
    assert_eq!(Some('o'), t.next());
    assert_eq!(Some(' '), t.peek());
    assert_eq!(Some(' '), t.next());
    assert_eq!(Some('b'), t.peek());
    assert_eq!(Some('b'), t.next());
    assert_eq!(Some('a'), t.peek());
    assert_eq!(Some('a'), t.next());
    assert_eq!(Some('r'), t.peek());
    assert_eq!(Some('r'), t.next());
    assert_eq!(None, t.next());
    assert_eq!(None, t.next());
  }
  
  #[test]
  fn next_token() {
    let s = r#"Hello 122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Ident, "Hello", 0..5)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 5..6)), t.token());
    assert_eq!(Ok(Token::new(TType::Number, "122", 6..9)), t.token());
    
    let s = r#"Hello=122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Ident, "Hello", 0..5)), t.token());
    assert_eq!(Ok(Token::new(TType::Assign, "=", 5..6)), t.token());
    assert_eq!(Ok(Token::new(TType::Number, "122", 6..9)), t.token());
    
    let s = r#"+-*/%"#; // consuming operators is greedy
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Operator, "+-*/%", 0..5)), t.token());
    
    let s = r#"Hello    = 122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Ident, "Hello", 0..5)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, "    ", 5..9)), t.token());
    assert_eq!(Ok(Token::new(TType::Assign, "=", 9..10)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 10..11)), t.token());
    assert_eq!(Ok(Token::new(TType::Number, "122", 11..14)), t.token());
    
    let s = r#"Hello? = 122 kg"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Ident, "Hello", 0..5)), t.token());
    assert_eq!(Ok(Token::new(TType::Verbatim, "? ", 5..7)), t.token());
    assert_eq!(Ok(Token::new(TType::Assign, "=", 7..8)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 8..9)), t.token());
    assert_eq!(Ok(Token::new(TType::Number, "122", 9..12)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 12..13)), t.token());
    assert_eq!(Ok(Token::new(TType::Ident, "kg", 13..15)), t.token());
    
    let s = r#"Hello, there, Mr.=122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Ident, "Hello", 0..5)), t.token());
    assert_eq!(Ok(Token::new(TType::Verbatim, ", ", 5..7)), t.token());
    assert_eq!(Ok(Token::new(TType::Ident, "there", 7..12)), t.token());
    assert_eq!(Ok(Token::new(TType::Verbatim, ", ", 12..14)), t.token());
    assert_eq!(Ok(Token::new(TType::Ident, "Mr", 14..16)), t.token());
    assert_eq!(Ok(Token::new(TType::Verbatim, ".", 16..17)), t.token());
    assert_eq!(Ok(Token::new(TType::Assign, "=", 17..18)), t.token());
    assert_eq!(Ok(Token::new(TType::Number, "122", 18..21)), t.token());
    
    let s = r#"a + (1 * b)"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token::new(TType::Ident, "a", 0..1)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 1..2)), t.token());
    assert_eq!(Ok(Token::new(TType::Operator, "+", 2..3)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 3..4)), t.token());
    assert_eq!(Ok(Token::new(TType::LParen, "(", 4..5)), t.token());
    assert_eq!(Ok(Token::new(TType::Number, "1", 5..6)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 6..7)), t.token());
    assert_eq!(Ok(Token::new(TType::Operator, "*", 7..8)), t.token());
    assert_eq!(Ok(Token::new(TType::Whitespace, " ", 8..9)), t.token());
    assert_eq!(Ok(Token::new(TType::Ident, "b", 9..10)), t.token());
    assert_eq!(Ok(Token::new(TType::RParen, ")", 10..11)), t.token());
  }
}

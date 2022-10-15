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
  Ident,
  Number,
  String,
  Operator,
  Symbol,
  End,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
  pub ttype: TType,
  pub ttext: String,
}

impl Token {
  pub fn styled(&self) -> Option<String> {
    let ttext: &str = &self.ttext;
    match self.ttype {
      TType::Verbatim => Some(format!("{}", ttext.reset())),
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
  index: usize,
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
      self.peek = self.data.next();
    }
    self.peek
  }
  
  pub fn skip(&mut self) {
    self.peek = self.data.next();
    self.index = self.index + 1;
  }
  
  pub fn next(&mut self) -> Option<char> {
    let n = self.peek();
    if n != None {
      self.peek = self.data.next();
      self.index = self.index + 1;
    }
    n
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
      Ok(Token{ttype: TType::End, ttext: "".to_string()})
    }
  }
  
  fn push(&mut self, tok: Token) {
    self.tokens.push(tok);
  }
  
  fn scan(&mut self) -> Result<(), error::Error> {
    if let Some(c) = self.peek() {
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
      }
    }
    Err(error::Error::TokenNotMatched)
  }
  
  fn scan_verbatim(&mut self) -> Result<(), error::Error> {
    let mut buf = String::new();
    loop {
      if let Some(c) = self.peek() {
        if Self::is_ident_start(c) {
          break;
        }else if Self::is_number_start(c) {
          break;
        }else if Self::is_operator(c) {
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
    });
    Ok(())
  }
  
  fn scan_ident(&mut self) -> Result<(), error::Error> {
    let name = self.ident()?;
    self.push(Token{
      ttype: TType::Ident,
      ttext: name,
    });
    Ok(())
  }
  
  fn scan_number(&mut self) -> Result<(), error::Error> {
    let val = self.number()?;
    self.push(Token{
      ttype: TType::Number,
      ttext: val,
    });
    Ok(())
  }
  
  fn scan_operator(&mut self) -> Result<(), error::Error> {
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
    });
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
  
  fn is_operator(c: char) -> bool {
    c == EQUAL || c == ADD || c == SUB || c == MUL || c == DIV || c == MOD
  }
  
  fn ident(&mut self) -> Result<String, error::Error> {
    let mut buf = String::new();
    buf.push(self.assert_fn(|c| { Scanner::is_ident_start(c) })?);
    while let Some(c) = self.peek() {
      if Scanner::is_ident(c) {
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
    let mut t = super::Scanner::new(&s);
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
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "Hello".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: " ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Number, ttext: "122".to_string()}), t.token());
    
    let s = r#"Hello=122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "Hello".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Operator, ttext: "=".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Number, ttext: "122".to_string()}), t.token());
    
    let s = r#"=+-*/%"#; // consuming operators is greedy
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token{ttype: TType::Operator, ttext: "=+-*/%".to_string()}), t.token());
    
    let s = r#"Hello    = 122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "Hello".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: "    ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Operator, ttext: "=".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: " ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Number, ttext: "122".to_string()}), t.token());
    
    let s = r#"Hello? = 122 kg"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "Hello".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: "? ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Operator, ttext: "=".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: " ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Number, ttext: "122".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: " ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "kg".to_string()}), t.token());
    
    let s = r#"Hello, there, Mr.=122"#;
    let mut t = Scanner::new(s);
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "Hello".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: ", ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "there".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: ", ".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Ident, ttext: "Mr".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Verbatim, ttext: ".".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Operator, ttext: "=".to_string()}), t.token());
    assert_eq!(Ok(Token{ttype: TType::Number, ttext: "122".to_string()}), t.token());
  }
}

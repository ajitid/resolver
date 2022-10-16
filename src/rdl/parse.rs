use crate::rdl::scan::{Scanner, Token, TType};
use crate::rdl::exec::{Context, Node, Ident, Add};
use crate::rdl::unit;
use crate::rdl::error;

pub struct Parser<'a> {
  scan: &'a mut Scanner<'a>,
}

impl<'a> Parser<'a> {
  pub fn new(scan: &'a mut Scanner<'a>) -> Parser<'a> {
    Parser{
      scan: scan,
    }
  }
  
  pub fn parse(&'a mut self) -> Result<impl Node, error::Error> {
    let tok = self.scan.la();
    match tok {
      Some(tok) => Ok(self.parse_expr(tok)?),
      None => Err(error::Error::EndOfInput),
    }
  }
  
  fn parse_expr(&'a mut self, tok: &'a Token) -> Result<impl Node, error::Error> {
    match tok.ttype {
      TType::Ident => Ok(Ident::new(&tok.ttext)),
      TType::End => Err(error::Error::EndOfInput),
      _ => Err(error::Error::EndOfInput),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn exec_simple() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(1.0));
    cxt.set("c", unit::Unit::None(2.0));
    
    let t = r#"1 c"#;
    let n = (&mut Parser::new(&mut Scanner::new(t))).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(2.0)), n.exec(&cxt));
    
    let n = Add::new(Ident::new("a"), Ident::new("b"));
    assert_eq!(Ok(unit::Unit::None(2.0)), n.exec(&cxt));
    
    let n = Add::new(Ident::new("a"), Ident::new("c"));
    assert_eq!(Ok(unit::Unit::None(3.0)), n.exec(&cxt));
  }
  
  // #[test]
  // fn parse_simple() {
  //   let s = "a+b".to_string();
  //   let mut t = Scanner::new(&s);
  //   assert_eq!(Some('F'), t.peek());
  // }
  
}

use crate::rdl::scan::{Scanner, Token, TType};
use crate::rdl::exec::{Context, Node, Ident, Add};
use crate::rdl::unit;
use crate::rdl::error;

pub struct Parser;

impl Parser {
  pub fn new() -> Parser {
    Parser
  }
  
  pub fn parse<'a>(&'a self, scan: &'a mut Scanner<'a>) -> Result<impl Node, error::Error> {
    let tok = match scan.la() {
      Some(tok) => tok,
      None      => return Err(error::Error::EndOfInput),
    };
    Ok(self.parse_expr(scan)?)
  }
  
  fn parse_expr<'a>(&'a self, scan: &'a mut Scanner<'a>) -> Result<impl Node, error::Error> {
    if let Some(tok) = scan.la() {
      match tok.ttype {
        TType::Ident => Ok(Ident::new(&tok.ttext)),
        TType::End => Err(error::Error::EndOfInput),
        _ => Err(error::Error::EndOfInput),
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
  fn exec_simple() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(1.0));
    cxt.set("c", unit::Unit::None(2.0));
    
    let t = r#"1 c"#;
    let n = Parser::new().parse(&mut Scanner::new(t)).expect("Could not parse");
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

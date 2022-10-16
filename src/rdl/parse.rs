use crate::rdl::scan::{Scanner, TType};
use crate::rdl::exec::{Context, Node};
use crate::rdl::unit;
use crate::rdl::error;

pub struct Parser<'a> {
  scan: Scanner<'a>,
}

impl<'a> Parser<'a> {
  pub fn new(scan: Scanner<'a>) -> Parser<'a> {
    Parser{
      scan: scan,
    }
  }
  
  pub fn parse(&mut self) -> Result<Node, error::Error> {
    self.parse_arith()
  }
  
  fn parse_arith(&mut self) -> Result<Node, error::Error> {
    let left = self.parse_primary()?;
    
    let ttype = match self.scan.la_type() {
      Some(ttype) => ttype,
      None => return Ok(left),
    };
    if ttype != TType::Operator {
      return Ok(left);
    }
    let op = match self.scan.token() {
      Ok(op) => op,
      Err(err) => return Err(err.into()),
    };
    
    let ttype = match self.scan.la_type() {
      Some(ttype) => ttype,
      None => return Ok(left),
    };
    let right = match ttype {
      TType::Verbatim => return Ok(left),
      TType::End => return Ok(left),
      _ => self.parse_arith()?,
    };
    
    match op.ttext.chars().next().unwrap() {
      ADD => Ok(Node::new_add(left, right)),
      _ => Err(error::Error::TokenNotMatched),
    }
  }
  
  fn parse_primary(&mut self) -> Result<Node, error::Error> {
    let tok = match self.scan.token() {
      Ok(tok) => tok,
      Err(err) => return Err(err),
    };
    match tok.ttype {
      TType::Ident => Ok(Node::new_ident(&tok.ttext)),
      TType::Number => Ok(Node::new_number(tok.ttext.parse::<f64>()?)),
      TType::End => Err(error::Error::EndOfInput),
      _ => Err(error::Error::TokenNotMatched),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  // #[test]
  fn exec_simple() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(1.0));
    cxt.set("c", unit::Unit::None(2.0));
    
    let t = r#"1"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(1.0)), n.exec(&cxt));
    
    let t = r#"1+c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(2.0)), n.exec(&cxt));
    
    // let n = Node::new_add(Node::new_ident("a"), Node::new_ident("b"));
    // assert_eq!(Ok(unit::Unit::None(2.0)), n.exec(&cxt));
    
    // let n = Node::new_add(Node::new_ident("a"), Node::new_ident("c"));
    // assert_eq!(Ok(unit::Unit::None(3.0)), n.exec(&cxt));
  }
  
  // #[test]
  // fn parse_simple() {
  //   let s = "a+b".to_string();
  //   let mut t = Scanner::new(&s);
  //   assert_eq!(Some('F'), t.peek());
  // }
  
}

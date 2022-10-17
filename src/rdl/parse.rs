use crate::rdl::scan::{self, Scanner, TType};
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
    self.parse_arith_left(left)
  }
  
  fn parse_arith_left(&mut self, left: Node) -> Result<Node, error::Error> {
    self.scan.discard(TType::Whitespace);
    
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
    
    self.scan.discard(TType::Whitespace);
    
    let ttype = match self.scan.la_type() {
      Some(ttype) => ttype,
      None => return Ok(left),
    };
    let right = match ttype {
      TType::Verbatim => return Ok(left),
      TType::End => return Ok(left),
      TType::Ident => Some(self.parse_primary()?),
      TType::Number => Some(self.parse_primary()?),
      TType::LParen => Some(self.parse_primary()?),
      _ => None,
    };
    
    let opc = op.ttext.chars().next().unwrap();
    match right {
      Some(right) => match opc {
        scan::ADD => Ok(self.parse_arith_left(Node::new_add(left, right))?),
        scan::SUB => Ok(self.parse_arith_left(Node::new_sub(left, right))?),
        _ => Err(error::Error::TokenNotMatched),
      },
      None => match opc {
        scan::ADD => Ok(Node::new_add(left, self.parse_arith()?)),
        scan::SUB => Ok(Node::new_sub(left, self.parse_arith()?)),
        _ => Err(error::Error::TokenNotMatched),
      },
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
      TType::LParen => self.parse_expr(),
      TType::End => Err(error::Error::EndOfInput),
      _ => Err(error::Error::TokenNotMatched),
    }
  }
  
  fn parse_expr(&mut self) -> Result<Node, error::Error> {
    let expr = self.parse()?;
    let ttype = match self.scan.la_type() {
      Some(ttype) => ttype,
      None => return Err(error::Error::TokenNotMatched),
    };
    if ttype == TType::RParen {
      self.scan.token()?;
    }else{
      return Err(error::Error::TokenNotMatched);
    }
    Ok(expr)
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
    
    let t = r#"1"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(1.0)), n.exec(&cxt));
    
    let t = r#"1+c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(3.0)), n.exec(&cxt));
    
    let t = r#"1.25+c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(3.25)), n.exec(&cxt));
    
    let t = r#"1.25-c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(-0.75)), n.exec(&cxt));
    
    let t = r#"1.25 - c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(-0.75)), n.exec(&cxt));
    
    let t = r#"c - 1.25"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(0.75)), n.exec(&cxt));
    
    let t = r#"c - 1.25 + a"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(1.75)), n.exec(&cxt));
    
    let t = r#"c - (1.25 + a) + 10"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(9.75)), n.exec(&cxt));
    
    let t = r#"c - (1.25 + a) + 10, and then this text follows"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    assert_eq!(Ok(unit::Unit::None(9.75)), n.exec(&cxt));
    
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

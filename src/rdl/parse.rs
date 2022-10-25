use crate::rdl;
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
    self.scan.discard_fn(|ttype| {
      ttype == TType::Whitespace || ttype == TType::Verbatim
    });
    self.parse_enter()
  }
  
  fn parse_enter(&mut self) -> Result<Node, error::Error> {
    self.parse_assign()
  }
  
  fn parse_assign(&mut self) -> Result<Node, error::Error> {
    self.scan.discard(TType::Whitespace);

    let ttype = match self.scan.la_type() {
      Some(ttype) => ttype,
      None => return Err(error::Error::EndOfInput),
    };
    if ttype != TType::Ident {
      return self.parse_arith();
    }
    let left = match self.parse_ident() {
      Ok(left) => left,
      Err(err) => return Err(err.into()),
    };
    
    self.scan.discard(TType::Whitespace);
    
    let ttype = match self.scan.la_type() {
      Some(ttype) => ttype,
      None => return Ok(left),
    };
    if ttype != TType::Assign {
      return self.parse_arith_left(left);
    }
    let op = match self.scan.token() {
      Ok(op) => op,
      Err(err) => return Err(err.into()),
    };
    
    self.scan.discard(TType::Whitespace);
    
    let right = match self.parse_arith() {
      Ok(right) => right,
      Err(_) => return Ok(left),
    };
    
    Ok(Node::new_assign(left, right))
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
        scan::MUL => Ok(self.parse_arith_left(Node::new_mul(left, right))?),
        scan::DIV => Ok(self.parse_arith_left(Node::new_div(left, right))?),
        scan::MOD => Ok(self.parse_arith_left(Node::new_mod(left, right))?),
        _ => Err(error::Error::TokenNotMatched),
      },
      None => match opc {
        scan::ADD => Ok(Node::new_add(left, self.parse_arith()?)),
        scan::SUB => Ok(Node::new_sub(left, self.parse_arith()?)),
        scan::MUL => Ok(Node::new_mul(left, self.parse_arith()?)),
        scan::DIV => Ok(Node::new_div(left, self.parse_arith()?)),
        scan::MOD => Ok(Node::new_mod(left, self.parse_arith()?)),
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
  
  fn parse_ident(&mut self) -> Result<Node, error::Error> {
    let tok = match self.scan.token() {
      Ok(tok) => tok,
      Err(err) => return Err(err),
    };
    match tok.ttype {
      TType::Ident => Ok(Node::new_ident(&tok.ttext)),
      TType::End => Err(error::Error::EndOfInput),
      _ => Err(error::Error::TokenNotMatched),
    }
  }
  
  fn parse_expr(&mut self) -> Result<Node, error::Error> {
    let expr = self.parse_enter()?;
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
  
  fn parse_expr(t: &str) -> Result<Node, error::Error> {
    let n = Parser::new(Scanner::new(t)).parse()?;
    println!(">>> [{}] → [{}]", t, n);
    Ok(n)
  }
  
  fn exec_node(n: Node, cxt: &Context) -> Result<unit::Unit, error::Error> {
    let v = n.exec(cxt)?;
    println!("=== [{}] → {}", n, v);
    Ok(v)
  }
  
  fn exec_line(text: &str, cxt: &Context) -> String {
    let res = rdl::render(cxt, text);
    println!("*** [{}] → [{}]", text, res);
    res
  }
  
  #[test]
  fn parse_primitive() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(2.0));
    cxt.set("c", unit::Unit::None(3.0));
    
    let n = parse_expr(r#"1"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"123.456"#).expect("Could not parse");
    assert_eq!(Node::new_number(123.456), n);
    assert_eq!(Ok(unit::Unit::None(123.456)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"a"#).expect("Could not parse");
    assert_eq!(Node::new_ident("a"), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"Hello"#).expect("Could not parse");
    assert_eq!(Node::new_ident("Hello"), n);
    assert_eq!(Err(error::Error::UnboundVariable("Hello".to_string())), exec_node(n, &cxt));
  }
  
  #[test]
  fn parse_ws() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(2.0));
    cxt.set("c", unit::Unit::None(3.0));
    
    let n = parse_expr(r#"  1"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1  "#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"  1  "#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
  }
  
  #[test]
  fn parse_arith() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(2.0));
    cxt.set("c", unit::Unit::None(3.0));
    
    let n = parse_expr(r#"1 + 2"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Unit::None(3.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1 - 2"#).expect("Could not parse");
    assert_eq!(Node::new_sub(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Unit::None(-1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1 * 2"#).expect("Could not parse");
    assert_eq!(Node::new_mul(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Unit::None(2.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1 / 2"#).expect("Could not parse");
    assert_eq!(Node::new_div(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Unit::None(0.5)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"4 % 3"#).expect("Could not parse");
    assert_eq!(Node::new_mod(Node::new_number(4.0), Node::new_number(3.0)), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"a + 2"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_ident("a"), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Unit::None(3.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1 + b"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_number(1.0), Node::new_ident("b")), n);
    assert_eq!(Ok(unit::Unit::None(3.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"a + b"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_ident("a"), Node::new_ident("b")), n);
    assert_eq!(Ok(unit::Unit::None(3.0)), exec_node(n, &cxt));
  }
  
  #[test]
  fn parse_subexpr() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(2.0));
    cxt.set("c", unit::Unit::None(3.0));
    
    let n = parse_expr(r#"(1)"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"(a)"#).expect("Could not parse");
    assert_eq!(Node::new_ident("a"), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"((a))"#).expect("Could not parse");
    assert_eq!(Node::new_ident("a"), n);
    assert_eq!(Ok(unit::Unit::None(1.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"(1 + 2)"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Unit::None(3.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1 - 2 + 3"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_sub(Node::new_number(1.0), Node::new_number(2.0)), Node::new_number(3.0)), n);
    assert_eq!(Ok(unit::Unit::None(2.0)), exec_node(n, &cxt));
    
    let n = parse_expr(r#"1 - (2 + 3)"#).expect("Could not parse");
    assert_eq!(Node::new_sub(Node::new_number(1.0), Node::new_add(Node::new_number(2.0), Node::new_number(3.0))), n);
    assert_eq!(Ok(unit::Unit::None(-4.0)), n.exec(&cxt));
    
    let n = parse_expr(r#"1 - (2 + 3) / 4"#).expect("Could not parse");
    assert_eq!(Node::new_div(Node::new_sub(Node::new_number(1.0), Node::new_add(Node::new_number(2.0), Node::new_number(3.0))), Node::new_number(4.0)), n);
    assert_eq!(Ok(unit::Unit::None(-1.0)), n.exec(&cxt));
    
    let n = parse_expr(r#"1 - ((5 + 3) / 4)"#).expect("Could not parse");
    assert_eq!(Node::new_sub(Node::new_number(1.0), Node::new_div(Node::new_add(Node::new_number(5.0), Node::new_number(3.0)), Node::new_number(4.0))), n);
    assert_eq!(Ok(unit::Unit::None(-1.0)), n.exec(&cxt));
  }
  
  #[test]
  fn parse_in_context() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(2.0));
    cxt.set("c", unit::Unit::None(3.0));
    
    let t = r#"100+200; 0"#;
    assert_eq!("(100 + 200) => 300; 0 => 0", &exec_line(t, &cxt));
    
    let t = r#"100 + (b * 100), but 0 is 0"#;
    assert_eq!("(100 + (b * 100)) => 300; 0 => 0; 0 => 0", &exec_line(t, &cxt));
  }
  
  // #[test]
  fn parse_simple() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(1.0));
    cxt.set("c", unit::Unit::None(2.0));
    
    let t = r#"1"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(1.0)), n.exec(&cxt));
    
    let t = r#"1+c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(3.0)), n.exec(&cxt));
    
    let t = r#"1.25+c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(3.25)), n.exec(&cxt));
    
    let t = r#"1.25-c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(-0.75)), n.exec(&cxt));
    
    let t = r#"1.25 - c"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(-0.75)), n.exec(&cxt));
    
    let t = r#"c - 1.25"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(0.75)), n.exec(&cxt));
    
    let t = r#"c - 1.25 + a"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(1.75)), n.exec(&cxt));
    
    let t = r#"c - (1.25 + a)"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(-0.25)), n.exec(&cxt));
    
    let t = r#"c - (1.25 + a) + 10"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(9.75)), n.exec(&cxt));
    
    let t = r#"c - (1.25 + a) + 10, and then this text follows"#;
    let n = Parser::new(Scanner::new(t)).parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(9.75)), n.exec(&cxt));
    
    let t = r#"c - (1.25 + a) + 10 and then 20 - 10 - 1"#;
    let mut p = Parser::new(Scanner::new(t));
    let n = p.parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(9.75)), n.exec(&cxt));
    let n = p.parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Err(error::Error::UnboundVariable("and".to_string())), n.exec(&cxt));
    let n = p.parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Err(error::Error::UnboundVariable("then".to_string())), n.exec(&cxt));
    let n = p.parse().expect("Could not parse");
    println!(">>> [{}] → [{}]", t, n);
    assert_eq!(Ok(unit::Unit::None(9.0)), n.exec(&cxt));
    
    let t = r#"100+200;0"#;
    let mut p = Parser::new(Scanner::new(t));
    let n = p.parse().expect("Could not parse");
    println!(">>> [{}] => [{}]", t, n);
    
    let t = r#"100+200; 0"#;
    assert_eq!("(100 + 200) => 300; 0 => 0", &exec_line(t, &cxt))
  }
  
}

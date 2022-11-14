use std::fmt;
use std::ops;

use crate::rdl;
use crate::rdl::scan::{self, Scanner, TType};
use crate::rdl::exec::{Context, Node};
use crate::rdl::unit;
use crate::rdl::error;

#[derive(Debug, PartialEq)]
pub struct Expr {
  pub range: ops::Range<usize>,
  pub ast: Node,
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.ast.fmt(f)
  }
}

pub struct Parser<'a> {
  scan: Scanner<'a>,
}

impl<'a> Parser<'a> {
  pub fn new(scan: Scanner<'a>) -> Parser<'a> {
    Parser{
      scan: scan,
    }
  }
  
  pub fn parse(&mut self) -> Result<Expr, error::Error> {
    self.scan.discard_fn(|ttype| {
      ttype == TType::Whitespace ||
      ttype == TType::Verbatim
    });
    self.parse_enter()
  }
  
  fn parse_enter(&mut self) -> Result<Expr, error::Error> {
    self.parse_assign()
  }
  
  fn parse_assign(&mut self) -> Result<Expr, error::Error> {
    self.scan.discard(TType::Whitespace);
    
    let left = match self.parse_ident() {
      Ok(left) => left,
      Err(_)   => return self.parse_typecast(),
    };

    self.scan.discard(TType::Whitespace);
    
    match self.scan.expect_token(TType::Assign) {
      Ok(_)  => {},
      Err(_) => return self.parse_typecast_left(left),
    };
    
    self.scan.discard(TType::Whitespace);
    
    let right = match self.parse_typecast() {
      Ok(right) => right,
      Err(_)    => return self.parse_typecast_left(left),
    };
    
    Ok(Expr{
      range: left.range.start..right.range.end,
      ast: Node::new_assign(left.ast, right.ast),
    })
  }
  
  fn parse_typecast(&mut self) -> Result<Expr, error::Error> {
    match self.parse_arith() {
      Ok(left) => self.parse_typecast_left(left),
      Err(err) => Err(err.into()),
    }
  }
  
  fn parse_typecast_left(&mut self, left: Expr) -> Result<Expr, error::Error> {
    self.scan.discard(TType::Whitespace);
    
    match self.scan.expect_token(TType::Typecast) {
      Ok(_)  => {},
      Err(_) => return self.parse_arith_left(left),
    };
    
    self.scan.discard(TType::Whitespace);
    
    let unit = match self.parse_unit() {
      Ok(unit) => unit,
      Err(_)   => return Ok(left),
    };
    
    Ok(Expr{
      range: left.range.start..unit.range.end,
      ast: Node::new_typecast(left.ast, unit.ast),
    })
  }
  
  fn parse_arith(&mut self) -> Result<Expr, error::Error> {
    match self.parse_primary() {
      Ok(left) => self.parse_arith_left(left),
      Err(err) => Err(err.into()),
    }
  }
  
  fn parse_arith_left(&mut self, left: Expr) -> Result<Expr, error::Error> {
    self.scan.discard(TType::Whitespace);
    
    let op = match self.scan.expect_token(TType::Operator) {
      Ok(op) => op,
      Err(_) => return Ok(left),
    };
    
    self.scan.discard(TType::Whitespace);
    
    let ttype = match self.scan.la() {
      Some(ttype) => ttype,
      None => return Ok(left),
    };
    let right = match ttype {
      TType::Verbatim => return Ok(left),
      TType::End      => return Ok(left),
      TType::Ident    => Some(self.parse_primary()?),
      TType::Number   => Some(self.parse_primary()?),
      TType::LParen   => Some(self.parse_primary()?),
      _               => return Ok(left),
    };
    
    let opc = op.ttext.chars().next().unwrap();
    match right {
      Some(right) => match opc {
        scan::ADD => Ok(self.parse_arith_left(Expr{
          range: left.range.start..right.range.end,
          ast: Node::new_add(left.ast, right.ast)
        })?),
        scan::SUB => Ok(self.parse_arith_left(Expr{
          range: left.range.start..right.range.end,
          ast: Node::new_sub(left.ast, right.ast)
        })?),
        scan::MUL => Ok(self.parse_arith_left(Expr{
          range: left.range.start..right.range.end,
          ast: Node::new_mul(left.ast, right.ast)
        })?),
        scan::DIV => Ok(self.parse_arith_left(Expr{
          range: left.range.start..right.range.end,
          ast: Node::new_div(left.ast, right.ast)
        })?),
        scan::MOD => Ok(self.parse_arith_left(Expr{
          range: left.range.start..right.range.end,
          ast: Node::new_mod(left.ast, right.ast)
        })?),
        _ => Err(error::Error::TokenNotMatched),
      },
      None => {
        let right = self.parse_arith()?;
        match opc {
          scan::ADD => Ok(Expr{
            range: left.range.start..right.range.end,
            ast: Node::new_add(left.ast, right.ast),
          }),
          scan::SUB => Ok(Expr{
            range: left.range.start..right.range.end,
            ast: Node::new_sub(left.ast, right.ast),
          }),
          scan::MUL => Ok(Expr{
            range: left.range.start..right.range.end,
            ast: Node::new_mul(left.ast, right.ast),
          }),
          scan::DIV => Ok(Expr{
            range: left.range.start..right.range.end,
            ast: Node::new_div(left.ast, right.ast),
          }),
          scan::MOD => Ok(Expr{
            range: left.range.start..right.range.end,
            ast: Node::new_mod(left.ast, right.ast),
          }),
          _ => Err(error::Error::TokenNotMatched),
        }
      },
    }
  }
  
  fn parse_primary(&mut self) -> Result<Expr, error::Error> {
    let tok = self.scan.expect_token_fn(|tok| {
      tok.ttype == TType::Ident  ||
      tok.ttype == TType::Number ||
      tok.ttype == TType::LParen
    })?;
    
    let rng = tok.range.clone();
    let exp = match &tok.ttype {
      TType::Ident  => Expr{
        range: tok.range,
        ast: Node::new_ident(&tok.ttext),
      },
      TType::Number => Expr{
        range: tok.range,
        ast: Node::new_number(tok.ttext.parse::<f64>()?),
      },
      TType::LParen => {
        let exp = self.parse_expr()?;
        Expr{
          range: tok.range.start..exp.range.end,
          ast: exp.ast,
        }
      },
      _ => return Err(error::Error::TokenNotMatched),
    };
    
    self.scan.discard(TType::Whitespace);
    
    match self.parse_unit() {
      Ok(unit) => Ok(Expr{
        range: rng.start..unit.range.end,
        ast: Node::new_typecast(exp.ast, unit.ast),
      }),
      Err(_) => Ok(exp),
    }
  }
  
  fn parse_expr(&mut self) -> Result<Expr, error::Error> {
    let expr = self.parse_enter()?;
    let tok = self.scan.expect_token(TType::RParen)?;
    Ok(Expr{
      range: expr.range.start..tok.range.end,
      ast: expr.ast,
    })
  }
  
  fn parse_ident(&mut self) -> Result<Expr, error::Error> {
    let tok = self.scan.expect_token(TType::Ident)?;
    Ok(Expr{
      range: tok.range,
      ast: Node::new_ident(&tok.ttext),
    })
  }
  
  fn parse_unit(&mut self) -> Result<Expr, error::Error> {
    let tok = self.scan.expect_token_fn(|tok| {
      tok.ttype == TType::Ident && if let Some(_) = unit::Unit::from(&tok.ttext) { true } else { false }
    })?;
    Ok(Expr{
      range: tok.range,
      ast: Node::new_ident(&tok.ttext),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  fn parse_expr(t: &str) -> Result<Node, error::Error> {
    let e = Parser::new(Scanner::new(t)).parse()?;
    println!(">>> [{}] → [{}]", t, e.ast);
    Ok(e.ast)
  }
  
  fn exec_node(n: Node, cxt: &mut Context) -> Result<unit::Value, error::Error> {
    let v = n.exec(cxt)?;
    println!("=== [{}] → {}", n, v);
    Ok(v)
  }
  
  fn exec_line(text: &str, cxt: &mut Context) -> String {
    let (_, res) = rdl::render_with_options(cxt, text, 0, 0, None, Some(&rdl::Options{verbose: true, debug: false}));
    println!("*** [{}] → [{}]", text, res.text());
    res.text().to_owned()
  }
  
  #[test]
  fn parse_primitive() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = parse_expr(r#"1"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"123.456"#).expect("Could not parse");
    assert_eq!(Node::new_number(123.456), n);
    assert_eq!(Ok(unit::Value::raw(123.456)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"a"#).expect("Could not parse");
    assert_eq!(Node::new_ident("a"), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"Hello"#).expect("Could not parse");
    assert_eq!(Node::new_ident("Hello"), n);
    assert_eq!(Err(error::Error::UnboundVariable("Hello".to_string())), exec_node(n, &mut cxt));
  }
  
  #[test]
  fn parse_ws() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = parse_expr(r#"  1"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1  "#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"  1  "#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
  }
  
  #[test]
  fn parse_arith() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = parse_expr(r#"1 + 2"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Value::raw(3.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 - 2"#).expect("Could not parse");
    assert_eq!(Node::new_sub(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Value::raw(-1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 * 2"#).expect("Could not parse");
    assert_eq!(Node::new_mul(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Value::raw(2.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 / 2"#).expect("Could not parse");
    assert_eq!(Node::new_div(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Value::raw(0.5)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"4 % 3"#).expect("Could not parse");
    assert_eq!(Node::new_mod(Node::new_number(4.0), Node::new_number(3.0)), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"a + 2"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_ident("a"), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Value::raw(3.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 + b"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_number(1.0), Node::new_ident("b")), n);
    assert_eq!(Ok(unit::Value::raw(3.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"a + b"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_ident("a"), Node::new_ident("b")), n);
    assert_eq!(Ok(unit::Value::raw(3.0)), exec_node(n, &mut cxt));
  }
  
  #[test]
  fn parse_subexpr() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = parse_expr(r#"(1)"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"(a)"#).expect("Could not parse");
    assert_eq!(Node::new_ident("a"), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"((a))"#).expect("Could not parse");
    assert_eq!(Node::new_ident("a"), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"(1 + 2)"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_number(1.0), Node::new_number(2.0)), n);
    assert_eq!(Ok(unit::Value::raw(3.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 - 2 + 3"#).expect("Could not parse");
    assert_eq!(Node::new_add(Node::new_sub(Node::new_number(1.0), Node::new_number(2.0)), Node::new_number(3.0)), n);
    assert_eq!(Ok(unit::Value::raw(2.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 - (2 + 3)"#).expect("Could not parse");
    assert_eq!(Node::new_sub(Node::new_number(1.0), Node::new_add(Node::new_number(2.0), Node::new_number(3.0))), n);
    assert_eq!(Ok(unit::Value::raw(-4.0)), n.exec(&mut cxt));
    
    let n = parse_expr(r#"1 - (2 + 3) / 4"#).expect("Could not parse");
    assert_eq!(Node::new_div(Node::new_sub(Node::new_number(1.0), Node::new_add(Node::new_number(2.0), Node::new_number(3.0))), Node::new_number(4.0)), n);
    assert_eq!(Ok(unit::Value::raw(-1.0)), n.exec(&mut cxt));
    
    let n = parse_expr(r#"1 - ((5 + 3) / 4)"#).expect("Could not parse");
    assert_eq!(Node::new_sub(Node::new_number(1.0), Node::new_div(Node::new_add(Node::new_number(5.0), Node::new_number(3.0)), Node::new_number(4.0))), n);
    assert_eq!(Ok(unit::Value::raw(-1.0)), n.exec(&mut cxt));
  }
  
  #[test]
  fn parse_assign() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = parse_expr(r#"d = 100"#).expect("Could not parse");
    assert_eq!(Node::new_assign(Node::new_ident("d"), Node::new_number(100.0)), n);
    assert_eq!(Ok(unit::Value::raw(100.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"d"#).expect("Could not parse");
    assert_eq!(Node::new_ident("d"), n); // value is now set for 'd'
    assert_eq!(Ok(unit::Value::raw(100.0)), exec_node(n, &mut cxt));
  }
  
  #[test]
  fn parse_unit_suffix() {
    let mut cxt = Context::new();
    cxt.set("kg", unit::Value::raw(4.0));
    
    let n = parse_expr(r#"kg"#).expect("Could not parse");
    assert_eq!(Node::new_ident("kg"), n);
    assert_eq!(Ok(unit::Value::raw(4.0)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"100 kg"#).expect("Could not parse");
    assert_eq!(Node::new_typecast(Node::new_number(100.0), Node::new_ident("kg")), n);
    assert_eq!(Ok(unit::Value::new(100.0, unit::Unit::Kilogram)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"(kg) kg"#).expect("Could not parse");
    assert_eq!(Node::new_typecast(Node::new_ident("kg"), Node::new_ident("kg")), n);
    assert_eq!(Ok(unit::Value::new(4.0, unit::Unit::Kilogram)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"1 ok"#).expect("Could not parse");
    assert_eq!(Node::new_number(1.0), n);
    assert_eq!(Ok(unit::Value::raw(1.0)), exec_node(n, &mut cxt));
  }
  
  #[test]
  fn parse_typecast() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = parse_expr(r#"100 kg in g"#).expect("Could not parse");
    assert_eq!(Node::new_typecast(Node::new_typecast(Node::new_number(100.0), Node::new_ident("kg")), Node::new_ident("g")), n);
    assert_eq!(Ok(unit::Value::new(100000.0, unit::Unit::Gram)), exec_node(n, &mut cxt));
    
    let n = parse_expr(r#"100 + 200 kg in g"#).expect("Could not parse");
    assert_eq!(Node::new_typecast(Node::new_add(Node::new_number(100.0), Node::new_typecast(Node::new_number(200.0), Node::new_ident("kg"))), Node::new_ident("g")), n);
    assert_eq!(Ok(unit::Value::new(300000.0, unit::Unit::Gram)), exec_node(n, &mut cxt));
  }
  
  #[test]
  fn parse_in_context() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let t = r#"100+200; 0"#;
    assert_eq!("(100 + 200) → 300; 0 → 0", &exec_line(t, &mut cxt));
    
    let t = r#"100 + (b * 100), but 0 is 0"#;
    assert_eq!("(100 + (b * 100)) → 300; 0 → 0; 0 → 0", &exec_line(t, &mut cxt));
  }
  
}

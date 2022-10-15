use crate::rdl::scan::{Scanner, Token, TType};
use crate::rdl::exec::{Context, Node, Ident, Add};
use crate::rdl::unit;
use crate::rdl::error;

pub fn parse<'a>(scan: &'a mut Scanner) -> Result<impl Node, error::Error> {
  match scan.token() {
    Ok(tok)  => Ok(parse_expr(scan, &tok)?),
    Err(err) => Err(err),
  }
}

fn parse_expr<'a>(scan: &'a Scanner, tok: &Token) -> Result<impl Node, error::Error> {
  match tok.ttype {
    TType::Ident => Ok(Ident::new(&tok.ttext)),
    TType::End => Err(error::Error::EndOfInput),
    _ => Err(error::Error::EndOfInput),
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

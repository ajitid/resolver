use std::collections::HashMap;

use crate::rdl::scan;
use crate::rdl::unit;
use crate::rdl::error;

pub struct Context {
  vars: HashMap<String, unit::Unit>,
}

impl Context {
  pub fn new() -> Context {
    Context{
      vars: HashMap::new(),
    }
  }
  
  pub fn set(&mut self, key: &str, val: unit::Unit) {
    self.vars.insert(key.to_string(), val);
  }
  
  pub fn get(&self, key: &str) -> Option<unit::Unit> {
    match self.vars.get(key) {
      Some(v) => Some(*v),
      None => None,
    }
  }
}

pub trait Node {
  fn exec(&self, cxt: &Context) -> Result<unit::Unit, error::Error>;
}

pub struct Ident {
  name: String,
}

impl Ident {
  pub fn new(name: &str) -> Ident {
    Ident{
      name: name.to_string(),
    }
  }
}

impl Node for Ident {
  fn exec(&self, cxt: &Context) -> Result<unit::Unit, error::Error> {
    match cxt.get(&self.name) {
      Some(v) => Ok(v),
      None => Err(error::Error::UnboundVariable(self.name.to_owned())),
    }
  }
}

pub struct Add<L: Node, R: Node> {
  left: L,
  right: R,
}

impl<L: Node, R: Node> Node for Add<L, R> {
  fn exec(&self, cxt: &Context) -> Result<unit::Unit, error::Error> {
    let l = self.left.exec(cxt)?;
    let r = self.right.exec(cxt)?;
    Ok(l + r)
  }
}

fn parse() -> impl Node {
  Add::<Ident, Ident>{
    left: Ident::new("a"),
    right: Ident::new("b"),
  }
}

pub struct Parser;

impl Parser {
  pub fn parse<'a>(&self, scan: &'a scan::Scanner) -> Result<Option<String>, error::Error> {
    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn exec_simple() {
    let add = Add::<Ident, Ident>{
      left: Ident::new("a"),
      right: Ident::new("b"),
    };
    
    let mut cxt = Context::new();
    cxt.set("a", unit::Unit::None(1.0));
    cxt.set("b", unit::Unit::None(1.0));
    assert_eq!(Ok(unit::Unit::None(2.0)), add.exec(&cxt));
    
    let s = "a+b".to_string();
    let mut t = scan::Scanner::new(&s);
    assert_eq!(Some('F'), t.peek());
  }
  
  // #[test]
  // fn parse_simple() {
  //   let s = "a+b".to_string();
  //   let mut t = scan::Scanner::new(&s);
  //   assert_eq!(Some('F'), t.peek());
  // }
  
}

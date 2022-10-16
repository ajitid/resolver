use std::collections::HashMap;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NType {
  Ident,
  Number,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
}

pub struct Node<'a> {
  ntype: NType,
  left:  Option<&'a Node<'a>>,
  right: Option<&'a Node<'a>>,
  text:  Option<String>,
  value: Option<f64>,
}

impl<'a> Node<'a> {
  pub fn new_ident(name: &str) -> Node<'a> {
    Node{
      ntype: NType::Ident,
      left: None, right: None,
      text: Some(name.to_string()),
      value: None,
    }
  }
  
  pub fn new_number(value: f64) -> Node<'a> {
    Node{
      ntype: NType::Number,
      left: None, right: None,
      text: None,
      value: Some(value),
    }
  }
  
  pub fn new_add(left: &'a Node, right: &'a Node) -> Node<'a> {
    Node{
      ntype: NType::Add,
      left: Some(left), right: Some(right),
      text: None,
      value: None,
    }
  }
  
  pub fn exec(&self, cxt: &Context) -> Result<unit::Unit, error::Error> {
    match self.ntype {
      Ident  => self.exec_ident(cxt),
      Number => self.exec_number(cxt),
      Add    => self.exec_add(cxt),
      _ => Err(error::Error::InvalidASTNode),
    }
  }
  
  fn exec_ident(&self, cxt: &Context) -> Result<unit::Unit, error::Error> {
    let name = match self.text {
      Some(name) => name,
      None => return Err(error::Error::InvalidASTNode),
    };
    match cxt.get(&name) {
      Some(v) => Ok(v),
      None => Err(error::Error::UnboundVariable(name.to_owned())),
    }
  }
  
  fn exec_number(&self, cxt: &Context) -> Result<unit::Unit, error::Error> {
    match self.value {
      Some(v) => Ok(unit::Unit::None(v)),
      None => Err(error::Error::InvalidASTNode),
    }
  }
  
  fn exec_add(&self, cxt: &Context) -> Result<unit::Unit, error::Error> {
    let left = match self.left {
      Some(left) => left,
      None => return Err(error::Error::InvalidASTNode),
    };
    let right = match self.right {
      Some(right) => right,
      None => return Err(error::Error::InvalidASTNode),
    };
    let left = left.exec(cxt)?;
    let right = right.exec(cxt)?;
    Ok(left + right)
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

    let n = Node::new_add(&Node::new_ident("a"), &Node::new_ident("b"));
    assert_eq!(Ok(unit::Unit::None(2.0)), n.exec(&cxt));
    
    let n = Node::new_add(&Node::new_ident("a"), &Node::new_ident("c"));
    assert_eq!(Ok(unit::Unit::None(3.0)), n.exec(&cxt));
  }
  
}

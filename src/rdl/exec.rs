use std::fmt;
use std::collections::HashMap;

use crate::rdl::unit;
use crate::rdl::error;

pub struct Context {
  vars: HashMap<String, unit::Value>,
}

impl Context {
  pub fn new() -> Context {
    Context{
      vars: HashMap::new(),
    }
  }
  
  pub fn new_with_stdlib() -> Context {
    let mut vars = HashMap::new();
    vars.insert("pi".to_string(), unit::Value::raw(std::f64::consts::PI));
    Context{
      vars: vars,
    }
  }
  
  pub fn set(&mut self, key: &str, val: unit::Value) {
    self.vars.insert(key.to_string(), val);
  }
  
  pub fn get(&self, key: &str) -> Option<unit::Value> {
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
  Assign,
  Typecast,
  Add,
  Sub,
  Mul,
  Div,
  Mod,
}

impl fmt::Display for NType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      NType::Ident    => write!(f, "ident"),
      NType::Number   => write!(f, "value"),
      NType::Assign   => write!(f, "="),
      NType::Typecast => write!(f, ":"),
      NType::Add      => write!(f, "+"),
      NType::Sub      => write!(f, "-"),
      NType::Mul      => write!(f, "*"),
      NType::Div      => write!(f, "/"),
      NType::Mod      => write!(f, "%"),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct Node {
  ntype: NType,
  left:  Option<Box<Node>>,
  right: Option<Box<Node>>,
  text:  Option<String>,
  value: Option<f64>,
}

impl fmt::Display for Node {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.print() {
      Ok(out)  => write!(f, "{}", out),
      Err(err) => write!(f, "error: {}", err),
    }
  }
}

impl Node {
  pub fn new_ident(name: &str) -> Node {
    Node{
      ntype: NType::Ident,
      left: None, right: None,
      text: Some(name.to_string()),
      value: None,
    }
  }
  
  pub fn new_number(value: f64) -> Node {
    Node{
      ntype: NType::Number,
      left: None, right: None,
      text: None,
      value: Some(value),
    }
  }
  
  pub fn new_assign(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Assign,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some("=".to_string()),
      value: None,
    }
  }
  
  pub fn new_typecast(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Typecast,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some(":".to_string()),
      value: None,
    }
  }
  
  pub fn new_add(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Add,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some("+".to_string()),
      value: None,
    }
  }
  
  pub fn new_sub(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Sub,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some("-".to_string()),
      value: None,
    }
  }
  
  pub fn new_mul(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Mul,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some("*".to_string()),
      value: None,
    }
  }
  
  pub fn new_div(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Div,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some("/".to_string()),
      value: None,
    }
  }
  
  pub fn new_mod(left: Node, right: Node) -> Node {
    Node{
      ntype: NType::Mod,
      left: Some(Box::new(left)), right: Some(Box::new(right)),
      text: Some("%".to_string()),
      value: None,
    }
  }
  
  fn text<'a>(&'a self) -> Result<&'a str, error::Error> {
    match &self.text {
      Some(text) => Ok(text),
      None => Err(error::Error::InvalidASTNode(format!("{}: Expected text", self.ntype))),
    }
  }
  
  fn value(&self) -> Result<unit::Value, error::Error> {
    match self.value {
      Some(value) => Ok(unit::Value::raw(value)),
      None => Err(error::Error::InvalidASTNode(format!("{}: Expected value", self.ntype))),
    }
  }
  
  fn left<'a>(&'a self) -> Result<&'a Box<Node>, error::Error> {
    match &self.left {
      Some(left) => Ok(left),
      None => Err(error::Error::InvalidASTNode(format!("{}: Expected left child", self.ntype))),
    }
  }
  
  fn right<'a>(&'a self) -> Result<&'a Box<Node>, error::Error> {
    match &self.right {
      Some(right) => Ok(right),
      None => Err(error::Error::InvalidASTNode(format!("{}: Expected right child", self.ntype))),
    }
  }
  
  pub fn exec(&self, cxt: &mut Context) -> Result<unit::Value, error::Error> {
    match self.ntype {
      NType::Ident    => self.exec_ident(cxt),
      NType::Number   => self.exec_number(cxt),
      NType::Assign   => self.exec_assign(cxt),
      NType::Typecast => self.exec_typecast(cxt),
      NType::Add | NType::Sub | NType::Mul | NType::Div | NType::Mod => self.exec_arith(cxt),
    }
  }
  
  fn exec_ident(&self, cxt: &mut Context) -> Result<unit::Value, error::Error> {
    let name = self.text()?;
    match cxt.get(&name) {
      Some(v) => Ok(v),
      None => Err(error::Error::UnboundVariable(name.to_owned())),
    }
  }
  
  fn exec_number(&self, _cxt: &mut Context) -> Result<unit::Value, error::Error> {
    self.value()
  }
  
  fn exec_assign(&self, cxt: &mut Context) -> Result<unit::Value, error::Error> {
    let left = self.left()?;
    let right = self.right()?;
    let ident = match left.ntype {
      NType::Ident => left.text()?,
      _ => return Err(error::Error::InvalidASTNode(format!("{}: Expected identifier as left child, got: {}", self.ntype, left.ntype))),
    };
    let right = match right.exec(cxt) {
      Ok(right) => right,
      Err(err) => return Err(error::Error::InvalidASTNode(format!("{}: Could not exec right: {}", self.ntype, err))),
    };
    cxt.set(ident, right);
    Ok(right)
  }
  
  fn exec_typecast(&self, cxt: &mut Context) -> Result<unit::Value, error::Error> {
    let left = self.left()?;
    let right = self.right()?;
    let tcast = match right.ntype {
      NType::Ident => right.text()?,
      _ => return Err(error::Error::InvalidASTNode(format!("{}: Expected identifier as right child, got: {}", self.ntype, right.ntype))),
    };
    let left = match left.exec(cxt) {
      Ok(left) => left,
      Err(err) => return Err(error::Error::InvalidASTNode(format!("{}: Could not exec left: {}", self.ntype, err))),
    };
    // let res = match unit::Value::raw(1.0, Some(tcast)) {
    //   Some(res) => res,
    //   None => return Err(error::Error::InvalidASTNode(format!("{}: No such type: {}", self.ntype, tcast))),
    // };
    Ok(left) // ignore cast for now, just use the main expression
  }
  
  fn exec_arith(&self, cxt: &mut Context) -> Result<unit::Value, error::Error> {
    let left = match self.left()?.exec(cxt) {
      Ok(left) => left,
      Err(err) => return Err(error::Error::InvalidASTNode(format!("{}: Could not exec left: {}", self.ntype, err))),
    };
    let right = match self.right()?.exec(cxt) {
      Ok(right) => right,
      Err(err) => return Err(error::Error::InvalidASTNode(format!("{}: Could not exec right: {}", self.ntype, err))),
    };
    match self.ntype {
      NType::Add => Ok(left + right),
      NType::Sub => Ok(left - right),
      NType::Mul => Ok(left * right),
      NType::Div => Ok(left / right),
      NType::Mod => Ok(left % right),
      _ => Err(error::Error::InvalidASTNode(format!("{}: Unsupported operation", self.ntype))),
    }
  }
  
  pub fn print(&self) -> Result<String, error::Error> {
    match self.ntype {
      NType::Ident    => self.print_ident(),
      NType::Number   => self.print_number(),
      NType::Assign   => self.print_assign(),
      NType::Typecast => self.print_typecast(),
      NType::Add | NType::Sub | NType::Mul | NType::Div | NType::Mod => self.print_arith(),
    }
  }
  
  fn print_ident(&self) -> Result<String, error::Error> {
    Ok(self.text()?.to_owned())
  }
  
  fn print_number(&self) -> Result<String, error::Error> {
    Ok(format!("{}", self.value()?))
  }
  
  fn print_arith(&self) -> Result<String, error::Error> {
    Ok(format!("({} {} {})", self.left()?.print()?, self.ntype, self.right()?.print()?))
  }
  
  fn print_assign(&self) -> Result<String, error::Error> {
    Ok(format!("({} {} {})", self.left()?.print()?, self.ntype, self.right()?.print()?))
  }
  
  fn print_typecast(&self) -> Result<String, error::Error> {
    Ok(format!("{}({}))", self.right()?.print()?, self.left()?.print()?))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn exec_simple() {
    let mut cxt = Context::new();
    cxt.set("a", unit::Value::raw(1.0));
    cxt.set("b", unit::Value::raw(2.0));
    cxt.set("c", unit::Value::raw(3.0));
    
    let n = Node::new_ident("a");
    assert_eq!(Ok(unit::Value::raw(1.0)), n.exec(&mut cxt));
    
    let n = Node::new_number(1.25);
    assert_eq!(Ok(unit::Value::raw(1.25)), n.exec(&mut cxt));
    
    let n = Node::new_add(Node::new_ident("a"), Node::new_ident("b"));
    assert_eq!(Ok(unit::Value::raw(3.0)), n.exec(&mut cxt));
    
    let n = Node::new_sub(Node::new_ident("a"), Node::new_ident("c"));
    assert_eq!(Ok(unit::Value::raw(-2.0)), n.exec(&mut cxt));
    
    let n = Node::new_mul(Node::new_ident("a"), Node::new_ident("c"));
    assert_eq!(Ok(unit::Value::raw(3.0)), n.exec(&mut cxt));
    
    let n = Node::new_div(Node::new_ident("a"), Node::new_ident("b"));
    assert_eq!(Ok(unit::Value::raw(0.5)), n.exec(&mut cxt));
    
    let n = Node::new_mod(Node::new_ident("c"), Node::new_ident("b"));
    assert_eq!(Ok(unit::Value::raw(1.0)), n.exec(&mut cxt));
    
    let n = Node::new_assign(Node::new_ident("d"), Node::new_number(123.0));
    assert_eq!(Ok(unit::Value::raw(123.0)), n.exec(&mut cxt));
    
    let n = Node::new_typecast(Node::new_ident("d"), Node::new_ident("kg"));
    assert_eq!(Ok(unit::Value::raw(123.0)), n.exec(&mut cxt));
  }
  
}

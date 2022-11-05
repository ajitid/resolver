pub mod error;
pub mod scan;
pub mod parse;
pub mod exec;
pub mod unit;

use scan::Scanner;
use parse::Parser;
use exec::Context;

use crate::text::attrs;

pub fn render(cxt: &mut Context, text: &str) -> String {
  render_with_attrs(cxt, text, 0, 0, None).1.text().to_owned()
}

pub fn render_with_attrs(cxt: &mut Context, text: &str, boff0: usize, boff1: usize, attrs: Option<&Vec<attrs::Attributes>>) -> (attrs::Attributed, attrs::Attributed) {
  let mut g = String::new();
  let mut s0: Vec<attrs::Span> = Vec::new();
  let mut s1: Vec<attrs::Span> = Vec::new();
  let mut p = Parser::new(Scanner::new(text));
  let mut i = 0;
  loop {
    let exp = match p.parse() {
      Ok(exp) => exp,
      Err(_)  => break,
    };
    
    let res = match exp.ast.exec(cxt) {
      Ok(res) => res.to_string(),
      Err(_)  => continue,
    };
    
    if i > 0 {
      g.push_str("; ");
    }
    
    // g.push_str(&format!("({:?}) ", boff0+exp.range.start..boff0+exp.range.end));
    // g.push_str(&format!("{} → ", exp.ast));
    
    if let Some(attrs) = &attrs {
      let l = boff1 + g.len();
      let a = &attrs[i % attrs.len()];
      s0.push(attrs::Span::new(boff0+exp.range.start..boff0+exp.range.end, a.clone()));
      s1.push(attrs::Span::new(l..l+res.len(), a.clone()));
    }
    
    g.push_str(&res);
    // g.push_str(&format!("{}", res));
    
    i += 1;
  }
  (
    attrs::Attributed::new_with_str(text, s0),
    attrs::Attributed::new_with_string(g, s1),
  )
}

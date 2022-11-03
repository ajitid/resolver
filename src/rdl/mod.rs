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
  render_with_attrs(cxt, text, 0, None).text().to_owned()
}

pub fn render_with_attrs(cxt: &mut Context, text: &str, offset: usize, attrs: Option<&Vec<attrs::Attributes>>) -> attrs::Attributed {
  let mut g = String::new();
  let mut s: Vec<attrs::Span> = Vec::new();
  let mut p = Parser::new(Scanner::new(text));
  let mut i = 0;
  loop {
    let exp = match p.parse_with_meta() {
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
    
    g.push_str(&format!("({:?}) ", exp.range));
    g.push_str(&format!("{} → ", exp.ast));
    
    if let Some(attrs) = &attrs {
      let l = offset + g.len();
      s.push(attrs::Span::new(l..l+res.len(), attrs[i % attrs.len()].clone()));
    }
    
    g.push_str(&format!("{}", res));
    
    i += 1;
  }
  attrs::Attributed::new_with_string(g, s)
}

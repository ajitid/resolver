pub mod error;
pub mod scan;
pub mod parse;
pub mod exec;
pub mod unit;

use scan::Scanner;
use parse::Parser;
use exec::Context;

pub fn render(cxt: &Context, text: &str) -> String {
  let mut g = String::new();
  let mut p = Parser::new(Scanner::new(text));
  let mut i = 0;
  loop {
    let r = match p.parse() {
      Ok(r) => r,
      Err(_)  => break,
    };
    
    let res = match r.exec(&cxt) {
      Ok(res) => res,
      Err(_)  => continue,
    };
    
    if i > 0 { g.push_str("; "); }
    g.push_str(&format!("{}", r));
    g.push_str(&format!(" => {}", res));
    
    i += 1;
  }
  g
}

use std::ops;
use std::cmp::{min, max, Ordering};

use crossterm::style::{Stylize, Color};

#[derive(Debug, Clone, Copy)]
pub enum Mode {
  Terminal,
  Markup,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attributes {
  pub bold: bool,
  pub color: Option<Color>,
}

impl Attributes {
  pub fn render(&self, text: &str) -> String {
    self.render_with_mode(text, Mode::Terminal)
  }
  
  fn render_with_mode(&self, text: &str, mode: Mode) -> String {
    match mode {
      Mode::Terminal => self.render_term(text),
      Mode::Markup   => self.render_html(text),
    }
  }
  
  fn render_term(&self, text: &str) -> String {
    let mut styled = text.stylize();
    if self.bold {
      styled = styled.bold();
    }
    if let Some(color) = self.color {
      styled = styled.with(color);
    }
    styled.to_string()
  }
  
  fn render_html(&self, text: &str) -> String {
    let mut attrd = String::new();
    if self.bold {
      attrd.push_str("<b>");
    }
    if let Some(color) = self.color {
      attrd.push_str(&format!("<{:?}>", color));
    }
    attrd.push_str(text);
    if let Some(color) = self.color {
      attrd.push_str(&format!("</{:?}>", color));
    }
    if self.bold {
      attrd.push_str("</b>");
    }
    attrd
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Span {
  range: ops::Range<usize>,
  attrs: Attributes,
}

impl PartialOrd for Span {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Span {
  fn cmp(&self, other: &Self) -> Ordering {
    self.range.start.cmp(&other.range.start)
  }
}

impl Span {
  pub fn new(range: ops::Range<usize>, attrs: Attributes) -> Span {
    Span{
      range: range,
      attrs: attrs,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Attributed {
  text: String,
  spans: Vec<Span>,
}

impl Attributed {
  pub fn new() -> Attributed {
    Attributed{
      text: String::new(),
      spans: Vec::new(),
    }
  }
  
  pub fn new_with_str(text: &str, spans: Vec<Span>) -> Attributed {
    Attributed{
      text: text.to_string(),
      spans: spans,
    }
  }
  
  pub fn new_with_string(text: String, spans: Vec<Span>) -> Attributed {
    Attributed{
      text: text,
      spans: spans,
    }
  }
  
  pub fn len(&self) -> usize {
    self.text.len()
  }
  
  pub fn text<'a>(&'a self) -> &'a str {
    &self.text
  }
  
  pub fn spans<'a>(&'a self) -> &'a Vec<Span> {
    &self.spans
  }
  
  pub fn push(&mut self, another: &Attributed) {
    self.text.push_str(&another.text);
    for e in &another.spans {
      self.spans.push(e.clone());
    }
  }
  
  pub fn push_str(&mut self, text: &str) {
    self.text.push_str(text);
  }
  
  pub fn render(&self) -> String {
    self.render_with_mode(Mode::Terminal)
  }
  
  fn render_with_mode(&self, mode: Mode) -> String {
    render_with_mode(&self.text, &self.spans, mode)
  }
}

pub fn render(text: &str, spans: &Vec<Span>) -> String {
  render_with_mode(text, spans, Mode::Terminal)
}

pub fn render_with_offset(text: &str, boff: usize, spans: &Vec<Span>) -> String {
  render_with_options(text, boff, spans, Mode::Terminal)
}

fn render_with_mode(text: &str, spans: &Vec<Span>, mode: Mode) -> String {
  render_with_options(text, 0, spans, mode)
}

fn render_with_options(text: &str, boff: usize, spans: &Vec<Span>, mode: Mode) -> String {
  let mut dup = spans.clone();
  dup.sort();
  
  let len = text.len();
  let mut x = 0;
  let mut attrd = String::new();
  for span in dup {
    if span.range.end < boff { // skip spans that end before the current offset
      continue;
    }
    let start = min(max(boff, span.range.start) - boff, len);
    if start > x { // copy before span starts
      attrd.push_str(&text[x..start]);
    }
    let end = min(span.range.end - boff, len);
    if end > start { // copy attributed range
      attrd.push_str(&span.attrs.render_with_mode(&text[start..end], mode));
    }
    x = end;
  }
  if x < len {
    attrd.push_str(&text[x..]);
  }
  
  attrd
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn render_attributes() {
    let t = "Hello, there.";

    let a = vec![Span::new(0..5, Attributes{bold:true, color: None})];
    assert_eq!("<b>Hello</b>, there.", render_with_mode(t, &a, Mode::Markup));

    let a = vec![Span::new(0..5, Attributes{bold:true, color: Some(Color::Blue)})];
    assert_eq!("<b><Blue>Hello</Blue></b>, there.", render_with_mode(t, &a, Mode::Markup));

    let a = vec![Span::new(7..12, Attributes{bold:false, color: Some(Color::Green)}), Span::new(0..5, Attributes{bold:true, color: Some(Color::Blue)})];
    assert_eq!("<b><Blue>Hello</Blue></b>, <Green>there</Green>.", render_with_mode(t, &a, Mode::Markup));
  }
  
  #[test]
  fn render_attributes_with_offset() {
    let t = "Hello, there.";
    let x = 7;
    let p = &t[x..];
    
    let a = vec![Span::new(7..12, Attributes{bold:false, color: Some(Color::Green)}), Span::new(12..13, Attributes{bold:true, color: None})];
    assert_eq!("<Green>there</Green><b>.</b>", render_with_options(p, x, &a, Mode::Markup));
  }
  
  #[test]
  fn render_attributed() {
    let t = "Hello, there.";

    let a = Attributed::new_with_str(t, vec![Span::new(0..5, Attributes{bold:true, color: None})]);
    assert_eq!("<b>Hello</b>, there.", a.render_with_mode(Mode::Markup));

    let a = Attributed::new_with_str(t, vec![Span::new(0..5, Attributes{bold:true, color: None})]);
    assert_eq!("<b>Hello</b>, there.", a.render_with_mode(Mode::Markup));

    let a = Attributed::new_with_str(t, vec![Span::new(0..5, Attributes{bold:true, color: Some(Color::Blue)})]);
    assert_eq!("<b><Blue>Hello</Blue></b>, there.", a.render_with_mode(Mode::Markup));
    
    let a = Attributed::new_with_str(t, vec![
      Span::new(7..12, Attributes{bold:false, color: Some(Color::Green)}), // deliberately out of order
      Span::new(0..5, Attributes{bold:true, color: Some(Color::Blue)})
    ]);
    assert_eq!("<b><Blue>Hello</Blue></b>, <Green>there</Green>.", a.render_with_mode(Mode::Markup));
  }
  
}

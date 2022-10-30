use std::ops;
use std::cmp::{min, Ordering};

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
pub struct Attributed<'a> {
  text: &'a str,
  spans: Vec<Span>,
}

impl<'a> Attributed<'a> {
  pub fn new(text: &'a str, spans: Vec<Span>) -> Attributed<'a> {
    Attributed{
      text: text,
      spans: spans,
    }
  }
  
  pub fn render(&self) -> String {
    self.render_with_mode(Mode::Terminal)
  }
  
  fn render_with_mode(&self, mode: Mode) -> String {
    render_with_mode(self.text, &self.spans, mode)
  }
}

pub fn render(text: &str, spans: &Vec<Span>) -> String {
  render_with_mode(text, spans, Mode::Terminal)
}

fn render_with_mode(text: &str, spans: &Vec<Span>, mode: Mode) -> String {
  let mut dup = spans.clone();
  dup.sort();
  
  let len = text.len();
  let mut x = 0;
  let mut attrd = String::new();
  for span in dup {
    let start = min(span.range.start, len);
    if span.range.start > x { // copy before span starts
      attrd.push_str(&text[x..start]);
    }
    let end = min(span.range.end, len);
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
  fn render_attributed() {
    let t = "Hello, there.";

    let a = Attributed::new(t, vec![Span::new(0..5, Attributes{bold:true, color: None})]);
    assert_eq!("<b>Hello</b>, there.", a.render_with_mode(Mode::Markup));

    let a = Attributed::new(t, vec![Span::new(0..5, Attributes{bold:true, color: None})]);
    assert_eq!("<b>Hello</b>, there.", a.render_with_mode(Mode::Markup));

    let a = Attributed::new(t, vec![Span::new(0..5, Attributes{bold:true, color: Some(Color::Blue)})]);
    assert_eq!("<b><Blue>Hello</Blue></b>, there.", a.render_with_mode(Mode::Markup));
    
    let a = Attributed::new(t, vec![
      Span::new(7..12, Attributes{bold:false, color: Some(Color::Green)}),
      Span::new(0..5, Attributes{bold:true, color: Some(Color::Blue)})
    ]);
    assert_eq!("<b><Blue>Hello</Blue></b>, <Green>there</Green>.", a.render_with_mode(Mode::Markup));
  }
  
}

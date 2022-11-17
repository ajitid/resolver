pub mod attrs;
pub mod layout;
pub mod action;

use std::fmt;
use std::ops;
use std::str;
use std::cmp::{min, max};

use action::{Action, Movement, Operation};

use crate::buffer::Buffer;

pub const ZERO_POS: Pos = Pos{x: 0, y: 0, index: 0};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Pos {
  index: usize,
  pub x: usize,
  pub y: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Line {
  num:   usize,
  coff:  usize, // line absolute lower bound, in chars
  boff:  usize, // line absolute lower bound, in bytes
  cext:  usize, // line absolute upper bound, in chars
  bext:  usize, // line absolute upper bound, in bytes
  chars: usize, // visual width, in chars
  bytes: usize, // visual width, in bytes
  hard:  bool,  // does this line break at a literal newline?
}

impl Line {
  pub fn text<'a>(&self, text: &'a str) -> &'a str {
    &text[self.boff..self.boff + self.bytes]
  }
  
  pub fn width(&self) -> usize {
    self.cext - self.coff
  }
  
  pub fn right(&self) -> usize {
    self.coff + self.chars
  }
  
  pub fn contains(&self, idx: usize) -> bool {
    idx >= self.coff && idx < self.cext
  }
  
  pub fn pos(&self, width: usize, idx: usize) -> Option<Pos> {
    if !self.contains(idx) {
      return None;
    }
    let eix = idx - self.coff;
    if eix < width {
      Some(Pos{index: idx, x: eix, y: self.num})
    }else{
      Some(Pos{index: idx, x: width, y: self.num}) // end of visual line
    }
  }
  
  pub fn debug_text<'a>(&self, text: &'a str, idx: usize) -> Option<String> {
    self.debug_text_range(text, ops::Range{start: idx, end: idx + 1})
  }
  
  pub fn debug_text_range<'a>(&self, text: &'a str, rng: ops::Range<usize>) -> Option<String> {
    if !self.contains(rng.start) {
      return None;
    }
    if rng.end < rng.start {
      return None;
    }
    let mut dbg = String::new();
    dbg.push_str(&text[self.boff..self.boff + self.bytes]);
    dbg.push('\n');
    dbg.push_str(&" ".repeat(rng.start - self.coff));
    dbg.push_str(&"^".repeat(rng.end - rng.start));
    Some(dbg)
  }
}

pub struct Lines<'a> {
  idx: usize,
  text: &'a str,
  metrics: &'a Vec<Line>,
}

impl<'a> Iterator for Lines<'a> {
  type Item = (&'a str, usize);
  
  fn next(&mut self) -> Option<Self::Item> {
    let n = self.metrics.len();
    if self.idx >= n {
      return None;
    }
    
    let loff = self.idx;
    let line = &self.metrics[self.idx];
    let boff = line.boff;
    let mut bext = line.bext;
    
    for _ in self.idx..self.metrics.len() {
      let line = &self.metrics[self.idx];
      bext = line.boff + line.bytes;
      self.idx += 1;
      if line.hard {
        break;
      }
    }
    
    Some((&self.text[boff..bext], self.idx - loff))
  }
}

pub trait Storage {
  fn width(&self) -> usize;
  fn num_lines(&self) -> usize;
  fn line_metrics<'a>(&'a self, i: usize) -> Option<&'a Line>;
  fn line_text<'a>(&'a self, i: usize) -> Option<&'a str>;
}

pub trait Renderable: Storage {
  fn write_line(&self, i: usize, b: &mut Buffer) -> (usize, usize);
  fn write_line_with_attrs(&self, i: usize, b: &mut Buffer, attrs: Option<&Vec<attrs::Span>>) -> (usize, usize);
}

pub struct Content {
  text: String,
  lines: Vec<Line>,
  spans: Option<Vec<attrs::Span>>,
  width: usize,
}

impl Content {
  pub fn new_with_str(text: &str, width: usize) -> Content {
    Self::new_with_string(text.to_owned(), width)
  }
  
  pub fn new_with_string(text: String, width: usize) -> Content {
    let lines = layout::layout(&text, width);
    Content{
      text: text,
      lines: lines,
      spans: None,
      width: width,
    }
  }
  
  pub fn new_with_attributed(text: String, spans: Vec<attrs::Span>, width: usize) -> Content {
    let lines = layout::layout(&text, width);
    Content{
      text: text,
      lines: lines,
      spans: Some(spans),
      width: width,
    }
  }
}

impl Storage for Content {
  fn width(&self) -> usize {
    self.width
  }
  
  fn num_lines(&self) -> usize {
    self.lines.len()
  }
  
  fn line_metrics<'a>(&'a self, i: usize) -> Option<&'a Line> {
    if i < self.lines.len() {
      Some(&self.lines[i])
    }else{
      None
    }
  }
  
  fn line_text<'a>(&'a self, i: usize) -> Option<&'a str> {
    match self.line_metrics(i) {
      Some(l) => Some(l.text(&self.text)),
      None => None,
    }
  }
}

impl Renderable for Content {
  fn write_line(&self, i: usize, b: &mut Buffer) -> (usize, usize) {
    match &self.spans {
      Some(spans) => self.write_line_with_attrs(i, b, Some(spans)),
      None => self.write_line_with_attrs(i, b, None),
    }
  }
  
  fn write_line_with_attrs(&self, i: usize, b: &mut Buffer, attrs: Option<&Vec<attrs::Span>>) -> (usize, usize) {
    let l = match self.line_metrics(i) {
      Some(l) => l,
      None => return (0, 0),
    };
    let t = l.text(&self.text);
    let t = match &attrs {
      Some(attrs) => attrs::render_with_offset(t, l.boff, attrs),
      None => t.to_string(),
    };
    b.push_str(&t);
    (l.chars, t.len())
  }
}

pub struct Text {
  text: String,
  width: usize,
  lines: Vec<Line>,
  spans: Option<Vec<attrs::Span>>,
  sel: Option<ops::Range<usize>>,
  loc: usize,
}

impl Text {
  pub fn new(width: usize) -> Text {
    Text{
      text: String::new(),
      width: width,
      lines: Vec::new(),
      spans: None,
      sel: None,
      loc: 0,
    }
  }
  
  pub fn new_with_str(width: usize, text: &str) -> Text {
    let mut c = Text{
      text: text.to_owned(),
      width: width,
      lines: Vec::new(),
      spans: None,
      sel: None,
      loc: 0,
    };
    c.reflow();
    c
  }
  
  pub fn len(&self) -> usize {
    match self.lines.len() {
      0 => 0,
      n => {
        let l = &self.lines[n-1];
        l.cext
      },
    }
  }
  
  pub fn paragraphs<'a>(&'a self) -> Lines<'a> {
    Lines{
      idx: 0,
      text: &self.text,
      metrics: &self.lines,
    }
  }
  
  fn line_with_index<'a>(&'a self, idx: usize) -> Option<&'a Line> {
    if self.lines.len() == 0 {
      return None;
    }
    for l in &self.lines {
      if idx >= l.coff && idx < l.cext {
        return Some(l);
      }
    }
    None
  }
  
  fn offset_for_index<'a>(&'a self, idx: usize) -> Option<usize> {
    let line = match self.line_with_index(idx) {
      Some(line) => line,
      None => return None,
    };
    
    let mut rem = idx - line.coff;
    if rem == 0 {
      return Some(line.boff);
    }
    
    let mut ecw = 0;
    for c in line.text(&self.text).chars() {
      ecw += c.len_utf8();
      rem -= 1;
      if rem == 0 {
        return Some(line.boff + ecw);
      }
    }
    
    Some(line.boff + line.bytes) // visual end of line
  }
  
  fn line_with_offset<'a>(&'a self, bix: usize) -> Option<&'a Line> {
    if self.lines.len() == 0 {
      return None;
    }
    for l in &self.lines {
      if bix >= l.boff && bix < l.bext {
        return Some(l);
      }
    }
    None
  }
  
  fn index_for_offset<'a>(&'a self, bix: usize) -> Option<usize> {
    let line = match self.line_with_offset(bix) {
      Some(line) => line,
      None => return None,
    };
    
    let mut rem = bix - line.boff;
    if rem == 0 {
      return Some(line.coff);
    }
    
    let mut ecw = 0;
    for c in line.text(&self.text).chars() {
      ecw += 1;
      rem -= c.len_utf8();
      if rem == 0 {
        return Some(line.coff + ecw);
      }
    }
    
    Some(line.coff + line.chars) // visual end of line
  }
  
  fn debug_text_for_index<'a>(&self, idx: usize) -> Option<String> {
    let line = match self.line_with_index(idx) {
      Some(line) => line,
      None => return None,
    };
    line.debug_text(&self.text, idx)
  }
  
  fn debug_text_for_range<'a>(&self, rng: ops::Range<usize>) -> Option<String> {
    let line = match self.line_with_index(rng.start) {
      Some(line) => line,
      None => return None,
    };
    line.debug_text_range(&self.text, rng)
  }
  
  fn next_offset<'a>(&'a self) -> usize {
    let n = self.lines.len();
    if n > 0 {
      self.lines[n-1].bext
    }else{
      0
    }
  }
  
  pub fn selection(&self) -> Option<ops::Range<usize>> {
    self.sel.clone()
  }
  
  pub fn selected_text<'a>(&'a self) -> Option<&'a str> {
    let sel = match &self.sel {
      Some(sel) => sel,
      None => return None,
    };
    let start = match self.offset_for_index(sel.start) {
      Some(bix) => bix,
      None => 0,
    };
    let end = match self.offset_for_index(sel.end) {
      Some(bix) => bix,
      None => self.len(),
    };
    Some(&self.text[start..end])
  }
  
  fn reflow(&mut self) -> &mut Self {
    self.lines = layout::layout(&self.text, self.width);
    self
  }
  
  pub fn edit(&mut self, idx: usize, action: Action) -> Option<Pos> {
    let dest = match self.to(idx, action.movement) {
      Some(dest) => dest,
      None => return None,
    };
    match action.operation {
      Operation::Move   => Some(dest), // nothing to do
      Operation::Select => self.select(Some(min(idx, dest.index)..max(idx, dest.index)), true),
      Operation::Delete => self.delete(min(idx, dest.index)..max(idx, dest.index)),
    }
  }
  
  pub fn edit_rel(&mut self, action: Action) -> Pos {
    let pos = match self.edit(self.loc, action) {
      Some(pos) => pos,
      None => self.index(self.loc),
    };
    self.loc = pos.index;
    pos
  }
  
  fn to(&self, idx: usize, mvmt: Movement) -> Option<Pos> {
    match mvmt {
      Movement::Up          => Some(self.up(idx)),
      Movement::Right       => Some(self.right(idx)),
      Movement::Down        => Some(self.down(idx)),
      Movement::Left        => Some(self.left(idx)),
      Movement::StartOfLine => Some(self.home(idx)),
      Movement::EndOfLine   => Some(self.end(idx)),
      Movement::Word        => self.find_fwd(idx+1, match_word),
      Movement::StartOfWord => if idx == 0 { None } else { self.find_rev(idx-1, match_word_boundary) },
      Movement::EndOfWord   => self.find_fwd(idx+1, match_word_boundary),
    }
  }
  
  fn to_rel(&mut self, movement: Movement) -> Pos {
    let pos = match self.to(self.loc, movement) {
      Some(pos) => pos,
      None => self.index(self.loc),
    };
    self.loc = pos.index;
    pos
  }
  
  fn to_abs(&mut self, idx: usize) -> Pos {
    let idx = if idx > self.len() {
      self.next_offset()
    }else{
      idx
    };
    let pos = self.index(idx);
    self.loc = idx;
    pos
  }
  
  fn find_fwd(&self, idx: usize, check: impl Fn(char, char) -> bool) -> Option<Pos> {
    let bix = match self.offset_for_index(idx) {
      Some(bix) => bix,
      None => return None,
    };
    let fwd = &self.text[bix..];
    let mut prev: char = '\0';
    let mut iter = fwd.chars();
    let mut coff = 0;
    loop {
      if let Some(c) = iter.next() {
        if check(c, prev) {
          return Some(self.index(idx + coff));
        }
        coff += 1;
        prev = c;
      }else{
        break;
      }
    }
    if check('\0', prev) {
      Some(self.index(idx + coff))
    }else{
      None
    }
  }
  
  fn find_rev(&self, idx: usize, check: impl Fn(char, char) -> bool) -> Option<Pos> {
    let bix = match self.offset_for_index(idx) {
      Some(bix) => bix,
      None => return None,
    };
    let rev = &self.text[..bix];
    let mut prev: char = '\0';
    let mut iter = rev.chars();
    let mut coff = 0;
    loop {
      if let Some(c) = iter.next_back() {
        if check(c, prev) {
          return Some(self.index(idx - coff));
        }
        coff += 1;
        prev = c;
      }else{
        break;
      }
    }
    if check('\0', prev) {
      Some(self.index(idx - coff))
    }else{
      None
    }
  }
  
  pub fn up(&self, idx: usize) -> Pos {
    let pos = self.index(idx);
    if pos.y == 0 {
      return ZERO_POS;
    }
    let n = pos.y - 1;
    let l = &self.lines[n];
    let w = l.chars;
    if w > pos.x {
      Pos{x: pos.x, y: n, index: l.coff + pos.x}
    }else{
      Pos{x: w, y: n, index: l.right()}
    }
  }
  
  pub fn up_rel(&mut self) -> Pos {
    let pos = self.up(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn down(&self, idx: usize) -> Pos {
    let nl = match self.lines.len() {
      0 => return ZERO_POS, // no line data; we have no content
      v => v,
    };
    let pos = self.index(idx);
    let y = if nl > 0 {
      min(nl - 1, pos.y)
    } else {
      0
    };
    let n = y + 1;
    if n >= nl {
      let l = &self.lines[y];
      if l.hard {
        return Pos{x: 0, y: y + 1, index: l.cext};
      }else{
        return Pos{x: l.chars, y: y, index: l.cext};
      }
    }
    let l = &self.lines[n];
    let w = l.chars;
    if w > pos.x {
      Pos{x: pos.x, y: n, index: l.coff + pos.x}
    }else{
      Pos{x: w, y: n, index: l.coff + l.chars}
    }
  }
  
  pub fn down_rel(&mut self) -> Pos {
    let pos = self.down(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn left(&self, idx: usize) -> Pos {
    if idx > 0 {
      self.index(idx - 1)
    }else{
      ZERO_POS
    }
  }
  
  pub fn left_rel(&mut self) -> Pos {
    let pos = self.left(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn right(&self, idx: usize) -> Pos {
    self.index(idx + 1)
  }
  
  pub fn right_rel(&mut self) -> Pos {
    let pos = self.right(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn home(&self, idx: usize) -> Pos {
    let nl = match self.lines.len() {
      0 => return ZERO_POS, // no line data; we have no content
      v => v,
    };
    let pos = self.index(idx);
    if pos.y >= nl {
      Pos{x: 0, y: nl, index: self.lines[nl - 1].cext}
    }else{
      Pos{x: 0, y: pos.y, index: self.lines[pos.y].coff}
    }
  }
  
  pub fn home_rel(&mut self) -> Pos {
    let pos = self.home(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn end(&self, idx: usize) -> Pos {
    let nl = match self.lines.len() {
      0 => return ZERO_POS, // no line data; we have no content
      v => v,
    };
    let pos = self.index(idx);
    if pos.y >= nl {
      Pos{x: 0, y: nl, index: self.lines[nl - 1].cext}
    } else {
      let l = &self.lines[pos.y];
      Pos{x: l.chars, y: pos.y, index: l.right()}
    }
  }
  
  pub fn end_rel(&mut self) -> Pos {
    let pos = self.end(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn index(&self, idx: usize) -> Pos {
    if self.len() == 0 {
      return ZERO_POS;
    }
    if idx == 0 {
      return ZERO_POS;
    }
    let idx = min(self.len(), idx);
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut hard: bool = false;
    for line in &self.lines {
      if let Some(pos) = line.pos(self.width, idx) {
        return pos;
      }
      y = line.num;
      x = line.width();
      hard = line.hard;
    }
    if hard || x + 1 > self.width {
      Pos{x: 0, y: self.lines.len(), index: idx}
    }else{
      Pos{x: x, y: y, index: idx}
    }
  }
  
  pub fn set_text(&mut self, text: String) {
    self.text = text;
    self.reflow();
  }
  
  pub fn insert(&mut self, idx: usize, c: char) -> Pos {
    let offset = match self.offset_for_index(idx) {
      Some(offset) => offset,
      None => self.next_offset(),
    };
    self.text.insert(offset, c);
    self.reflow();
    self.index(idx + 1)
  }
  
  pub fn insert_rel(&mut self, c: char) -> Pos {
    let pos = self.insert(self.loc, c);
    self.loc = pos.index;
    pos
  }
  
  pub fn delete(&mut self, rng: ops::Range<usize>) -> Option<Pos> {
    let start = match self.offset_for_index(rng.start) {
      Some(start) => start,
      None => return None,
    };
    let end = match self.offset_for_index(rng.end) {
      Some(end) => end,
      None => self.next_offset(),
    };
    self.text.replace_range(start..end, "");
    self.reflow();
    Some(self.index(start))
  }
  
  pub fn delete_rel(&mut self, rng: ops::Range<usize>) -> Pos {
    let pos = match self.delete(rng) {
      Some(pos) => pos,
      None => return self.index(self.loc),
    };
    self.loc = pos.index;
    pos
  }
  
  pub fn select(&mut self, rng: Option<ops::Range<usize>>, extend: bool) -> Option<Pos> {
    let rng = match rng {
      Some(rng) => rng,
      None => {
        self.sel = None;
        return None;
      },
    };
    
    let sel = if extend {
      match &self.sel {
        Some(sel) => min(sel.start, rng.start)..max(sel.end, rng.end),
        None => rng.clone(),
      }
    }else{
      rng.clone()
    };
    
    let dst = if rng.end > self.loc {
      rng.end
    }else{
      rng.start
    };
    
    self.sel = Some(sel);
    Some(self.index(dst))
  }
  
  pub fn select_rel(&mut self, rng: Option<ops::Range<usize>>, extend: bool) -> Pos {
    let pos = match self.select(rng, extend) {
      Some(pos) => pos,
      None => return self.index(self.loc),
    };
    self.loc = pos.index;
    pos
  }
  
  // TODO: deprecated below; these can be replaced by edit() operations.
  
  pub fn backspace(&mut self, idx: usize) -> Pos {
    let eix = idx - 1;
    let offset = match self.offset_for_index(eix) {
      Some(offset) => offset,
      None => return ZERO_POS,
    };
    self.text.remove(offset);
    self.reflow();
    self.index(eix)
  }
  
  pub fn backspace_rel(&mut self) -> Pos {
    if self.loc == 0 { // nothing to delete
      return ZERO_POS;
    }
    let pos = self.backspace(self.loc);
    self.loc = pos.index;
    pos
  }
}

impl Storage for Text {
  fn width(&self) -> usize {
    self.width
  }
  
  fn num_lines(&self) -> usize {
    self.lines.len()
  }
  
  fn line_metrics<'a>(&'a self, i: usize) -> Option<&'a Line> {
    if i < self.lines.len() {
      Some(&self.lines[i])
    }else{
      None
    }
  }
  
  fn line_text<'a>(&'a self, i: usize) -> Option<&'a str> {
    match self.line_metrics(i) {
      Some(l) => Some(l.text(&self.text)),
      None => None,
    }
  }
}

impl Renderable for Text {
  fn write_line(&self, i: usize, b: &mut Buffer) -> (usize, usize) {
    match &self.spans {
      Some(spans) => self.write_line_with_attrs(i, b, Some(spans)),
      None => self.write_line_with_attrs(i, b, None),
    }
  }
  
  fn write_line_with_attrs(&self, i: usize, b: &mut Buffer, attrs: Option<&Vec<attrs::Span>>) -> (usize, usize) {
    let l = match self.line_metrics(i) {
      Some(l) => l,
      None => return (0, 0),
    };
    let t = l.text(&self.text);
    let t = match &attrs {
      Some(attrs) => attrs::render_with_offset(t, l.boff, attrs),
      None => t.to_string(),
    };
    b.push_str(&t);
    (l.chars, t.len())
  }
}

impl fmt::Display for Text {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let n = self.num_lines();
    for i in 0..n {
      if let Some(l) = self.line_text(i) {
        write!(f, "{}\r\n", l)?;
      }
    }
    Ok(())
  }
}

fn match_word(curr: char, prev: char) -> bool {
  curr == '\0' || (!curr.is_whitespace() && prev.is_whitespace())
}

fn match_word_boundary(curr: char, prev: char) -> bool {
  (curr == '\0' || curr.is_whitespace()) && (prev == '\0' || !prev.is_whitespace())
}

#[cfg(test)]
mod tests {
  use super::*;
  
  macro_rules! test_reflow_case {
    ($width: expr, $text: expr, $ex_metrics: expr, $ex_lines: expr) => {
      let c = Text::new_with_str($width, $text);
      let actual = c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>();
      println!(">>> {:>3}w [{:?}] → {:?}", $width, $text, actual);
      assert_eq!($ex_metrics, c.lines);
      assert_eq!($ex_lines, actual);
    }
  }
  
  #[test]
  fn test_reflow() {
    test_reflow_case!(
      100, "😎",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 1, bext: 4, chars: 1, bytes: 4, hard: false,},
      ],
      vec![
        "😎",
      ]
    );
    
    test_reflow_case!(
      100, "Hello",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 5, bext: 5, chars: 5, bytes: 5, hard: false,},
      ],
      vec![
        "Hello",
      ]
    );
    
    test_reflow_case!(
      3, "Hello",
      vec![
          Line{num: 0, coff: 0, boff: 0, cext: 3, bext: 3, chars: 3, bytes: 3, hard: false},
          Line{num: 1, coff: 3, boff: 3, cext: 5, bext: 5, chars: 2, bytes: 2, hard: false},
      ],
      vec![
        "Hel",
        "lo",
      ]
    );
    
    test_reflow_case!(
      5, "😎 Hello",
      vec![
          Line{num: 0, coff: 0, boff: 0, cext: 2, bext: 5,  chars: 1, bytes: 4, hard: false},
          Line{num: 1, coff: 2, boff: 5, cext: 7, bext: 10, chars: 5, bytes: 5, hard: false},
      ],
      vec![
        "😎",
        "Hello",
      ]
    );
    
    test_reflow_case!(
      10, "Époustouflant",
      vec![
          Line{num: 0, coff: 0,  boff: 0,  cext: 10, bext: 11, chars: 10, bytes: 11, hard: false},
          Line{num: 1, coff: 10, boff: 11, cext: 13, bext: 14, chars: 3,  bytes: 3, hard: false},
      ],
      vec![
        "Époustoufl",
        "ant",
      ]
    );
    
    test_reflow_case!(
      8, "Hello there",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 6, bext: 6, chars: 5, bytes: 5, hard: false},
        Line{num: 1, coff: 6, boff: 6, cext: 11, bext: 11, chars: 5, bytes: 5, hard: false},
      ],
      vec![
        "Hello",
        "there",
      ]
    );
    
    test_reflow_case!(
      8, "Hello there monchambo",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 6, bext: 6, chars: 5, bytes: 5, hard: false},
        Line{num: 1, coff: 6, boff: 6, cext: 12, bext: 12, chars: 5, bytes: 5, hard: false},
        Line{num: 2, coff: 12, boff: 12, cext: 20, bext: 20, chars: 8, bytes: 8, hard: false},
        Line{num: 3, coff: 20, boff: 20, cext: 21, bext: 21, chars: 1, bytes: 1, hard: false},
      ],
      vec![
        "Hello",
        "there",
        "monchamb",
        "o",
      ]
    );
    
    test_reflow_case!(
      8, "Hello\nthere monchambo",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 6, bext: 6, chars: 5, bytes: 5, hard: true},
        Line{num: 1, coff: 6, boff: 6, cext: 12, bext: 12, chars: 5, bytes: 5, hard: false},
        Line{num: 2, coff: 12, boff: 12, cext: 20, bext: 20, chars: 8, bytes: 8, hard: false},
        Line{num: 3, coff: 20, boff: 20, cext: 21, bext: 21, chars: 1, bytes: 1, hard: false},
      ],
      vec![
        "Hello",
        "there",
        "monchamb",
        "o",
      ]
    );
    
    test_reflow_case!(
      100, "Hello\nthere.",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 6,  bext: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, coff: 6, boff: 6, cext: 12, bext: 12, chars: 6, bytes: 6, hard: false},
      ],
      vec![
        "Hello",
        "there.",
      ]
    );

    test_reflow_case!(
      100, "Hello 😎\nMonchambo.",
      vec![
        Line{num: 0, coff: 0, boff: 0,  cext:  8, bext: 11, chars: 7,  bytes: 10, hard: true},
        Line{num: 1, coff: 8, boff: 11, cext: 18, bext: 21, chars: 10, bytes: 10, hard: false},
      ],
      vec![
        "Hello 😎",
        "Monchambo.",
      ]
    );

    test_reflow_case!(
      100, "Hello\nthere.\n",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 6,  bext: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, coff: 6, boff: 6, cext: 13, bext: 13, chars: 6, bytes: 6, hard: true},
      ],
      vec![
        "Hello",
        "there.",
      ]
    );

    test_reflow_case!(
      100, "Hello\nthere.\n!",
      vec![
        Line{num: 0, coff: 0,  boff: 0,  cext: 6,  bext: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, coff: 6,  boff: 6,  cext: 13, bext: 13, chars: 6, bytes: 6, hard: true},
        Line{num: 2, coff: 13, boff: 13, cext: 14, bext: 14, chars: 1, bytes: 1, hard: false},
      ],
      vec![
        "Hello",
        "there.",
        "!",
      ]
    );
    
    test_reflow_case!(
      100, "Hello\n there.\n!",
      vec![
        Line{num: 0, coff: 0,  boff: 0,  cext: 6,  bext: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, coff: 6,  boff: 6,  cext: 14, bext: 14, chars: 7, bytes: 7, hard: true},
        Line{num: 2, coff: 14, boff: 14, cext: 15, bext: 15, chars: 1, bytes: 1, hard: false},
      ],
      vec![
        "Hello",
        " there.",
        "!",
      ]
    );
    
    test_reflow_case!(
      100, " \n \n \nHello.",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 2,  bext: 2,  chars: 1, bytes: 1, hard: true},
        Line{num: 1, coff: 2, boff: 2, cext: 4,  bext: 4,  chars: 1, bytes: 1, hard: true},
        Line{num: 2, coff: 4, boff: 4, cext: 6,  bext: 6,  chars: 1, bytes: 1, hard: true},
        Line{num: 3, coff: 6, boff: 6, cext: 12, bext: 12, chars: 6, bytes: 6, hard: false},
      ],
      vec![
        " ",
        " ",
        " ",
        "Hello.",
      ]
    );
    
    test_reflow_case!(
      100, "\n\n\nHello.",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 1, bext: 1, chars: 0, bytes: 0, hard: true},
        Line{num: 1, coff: 1, boff: 1, cext: 2, bext: 2, chars: 0, bytes: 0, hard: true},
        Line{num: 2, coff: 2, boff: 2, cext: 3, bext: 3, chars: 0, bytes: 0, hard: true},
        Line{num: 3, coff: 3, boff: 3, cext: 9, bext: 9, chars: 6, bytes: 6, hard: false},
      ],
      vec![
        "",
        "",
        "",
        "Hello.",
      ]
    );
    
    test_reflow_case!(
      100, "\nHello.\nOk",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 1,  bext: 1,  chars: 0, bytes: 0, hard: true},
        Line{num: 1, coff: 1, boff: 1, cext: 8,  bext: 8,  chars: 6, bytes: 6, hard: true},
        Line{num: 2, coff: 8, boff: 8, cext: 10, bext: 10, chars: 2, bytes: 2, hard: false},
      ],
      vec![
        "",
        "Hello.",
        "Ok",
      ]
    );
    
    test_reflow_case!(
      5, "\n\nHello.\nOk",
      vec![
        Line{num: 0, coff: 0, boff: 0, cext: 1,  bext: 1,  chars: 0, bytes: 0, hard: true},
        Line{num: 1, coff: 1, boff: 1, cext: 2,  bext: 2,  chars: 0, bytes: 0, hard: true},
        Line{num: 2, coff: 2, boff: 2, cext: 7,  bext: 7,  chars: 5, bytes: 5, hard: false},
        Line{num: 3, coff: 7, boff: 7, cext: 9,  bext: 9,  chars: 1, bytes: 1, hard: true},
        Line{num: 4, coff: 9, boff: 9, cext: 11, bext: 11, chars: 2, bytes: 2, hard: false},
      ],
      vec![
        "",
        "",
        "Hello",
        ".",
        "Ok",
      ]
    );
  }
  
  #[test]
  fn test_index() {
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Text::new_with_str(100, "").index(0));
    assert_eq!(Pos{index: 1, x: 1, y: 0}, Text::new_with_str(100, "H").index(1));
    assert_eq!(Pos{index: 2, x: 2, y: 0}, Text::new_with_str(100, "Hi").index(2));
    assert_eq!(Pos{index: 3, x: 0, y: 1}, Text::new_with_str(100, "Hi\n").index(3));
    assert_eq!(Pos{index: 4, x: 1, y: 1}, Text::new_with_str(100, "Hi\nT").index(4));
    assert_eq!(Pos{index: 5, x: 2, y: 1}, Text::new_with_str(100, "Hi\nTi").index(5));
    assert_eq!(Pos{index: 6, x: 3, y: 1}, Text::new_with_str(100, "Hi\nTim").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 2}, Text::new_with_str(100, "Hi\nTim\n").index(7));
    assert_eq!(Pos{index: 8, x: 1, y: 2}, Text::new_with_str(100, "Hi\nTim\n!").index(8));
    
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Text::new_with_str(100, "").index(0));
    assert_eq!(Pos{index: 1, x: 1, y: 0}, Text::new_with_str(100, "🎉").index(1));
    assert_eq!(Pos{index: 2, x: 2, y: 0}, Text::new_with_str(100, "🎉!").index(2));
    assert_eq!(Pos{index: 3, x: 0, y: 1}, Text::new_with_str(100, "🎉!\n").index(3));
    assert_eq!(Pos{index: 4, x: 1, y: 1}, Text::new_with_str(100, "🎉!\nT").index(4));
    assert_eq!(Pos{index: 5, x: 2, y: 1}, Text::new_with_str(100, "🎉!\nTi").index(5));
    assert_eq!(Pos{index: 6, x: 3, y: 1}, Text::new_with_str(100, "🎉!\nTim").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 2}, Text::new_with_str(100, "🎉!\nTim\n").index(7));
    assert_eq!(Pos{index: 8, x: 1, y: 2}, Text::new_with_str(100, "🎉!\nTim\n!").index(8));
    //
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Text::new_with_str(100, "Hello").index(4));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Hello!\n").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Text::new_with_str(100, "Hello!\n").index(7));
    
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Text::new_with_str(100, "Yo! 🤖").index(4));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Yo! 🤖!\n").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤖!\n").index(7));
  }
  
  #[test]
  fn test_movement_left() {
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello").left(0));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello").left(1));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello\n").left(6));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\nthere").left(7));

    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤪").left(0));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤪").left(1));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤪\n").left(6));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤪\nthere").left(7));
  }
  
  #[test]
  fn test_movement_right() {
    assert_eq!(Pos{index: 1,  x: 1, y: 0}, Text::new_with_str(100, "Hello").right(0));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello\n").right(4));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\n").right(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\n").right(6));
    
    assert_eq!(Pos{index: 1,  x: 1, y: 0}, Text::new_with_str(100, "Yo! 🤪").right(0));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤪\n").right(4));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤪\n").right(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤪\n").right(6));
  }
    
  #[test]
  fn test_movement_up() {
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello\n").up(5));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello\n").up(6));

    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤪\n").up(5));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤪\n").up(6));
    
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello,\nto\nyourself").up(7));
    assert_eq!(Pos{index: 1,  x: 1, y: 0}, Text::new_with_str(100, "Hello,\nto\nyourself").up(8));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").up(13));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").up(16));

    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤪,\nto\nyourself").up(7));
    assert_eq!(Pos{index: 1,  x: 1, y: 0}, Text::new_with_str(100, "Yo! 🤪,\nto\nyourself").up(8));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Yo! 🤪,\nto\nyourself").up(13));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Yo! 🤪,\nto\nyourself").up(16));
  }
  
  #[test]
  fn test_movement_down() {
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello").down(0));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello").down(1));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\n").down(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\n").down(6));
    
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤪").down(0));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤪").down(1));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤪\n").down(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤪\n").down(6));
    
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").down(2));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nZO\nyourself").down(6));
    assert_eq!(Pos{index: 18, x: 8, y: 2}, Text::new_with_str(100, "Hello,\nto\nyourself").down(18));
    
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Yo! 🤪,\nto\nyourself").down(2));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Yo! 🤪,\nZO\nyourself").down(6));
    assert_eq!(Pos{index: 18, x: 8, y: 2}, Text::new_with_str(100, "Yo! 🤪,\nto\nyourself").down(18));
  }
  
  #[test]
  fn test_movement_home() {
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello").home(0));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello").home(5));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello\n").home(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\n").home(6));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\nthere").home(6));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\nthere").home(99));
    
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤓").home(0));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤓").home(5));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Yo! 🤓\n").home(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤓\n").home(6));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤓\nthere").home(6));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤓\nthere").home(99));
  }
  
  #[test]
  fn test_movement_end() {
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello").end(0));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello").end(5));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Hello\n").end(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Hello\n").end(6));
    assert_eq!(Pos{index: 11, x: 5, y: 1}, Text::new_with_str(100, "Hello\nthere").end(6));
    assert_eq!(Pos{index: 11, x: 5, y: 1}, Text::new_with_str(100, "Hello\nthere").end(99));
    
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤓").end(0));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤓").end(5));
    assert_eq!(Pos{index: 5,  x: 5, y: 0}, Text::new_with_str(100, "Yo! 🤓\n").end(5));
    assert_eq!(Pos{index: 6,  x: 0, y: 1}, Text::new_with_str(100, "Yo! 🤓\n").end(6));
    assert_eq!(Pos{index: 11, x: 5, y: 1}, Text::new_with_str(100, "Yo! 🤓\nthere").end(6));
    assert_eq!(Pos{index: 11, x: 5, y: 1}, Text::new_with_str(100, "Yo! 🤓\nthere").end(99));
  }
  
  fn text_init(width: usize, text: &str) -> Text {
    let mut dst = Text::new(width);
    for c in text.chars() {
      dst.insert_rel(c);
    }
    dst
  }
  
  fn text_insert(into: &mut Text, text: &str) {
    for c in text.chars() {
      into.insert_rel(c);
    }
  }
  
  #[test]
  fn test_editing() {
    let mut t = Text::new(100);
    assert_eq!(Pos{index: 0, x: 0, y: 0}, t.backspace_rel());
    text_insert(&mut t, "Yo!!");
    assert_eq!(Pos{index: 3, x: 3, y: 0}, t.backspace_rel());
    t.insert_rel('\n');
    assert_eq!(Pos{index: 3, x: 3, y: 0}, t.backspace_rel());
    
    let mut t = Text::new(100);
    text_insert(&mut t, "Helll");
    t.backspace_rel();
    text_insert(&mut t, "o 😎 dude\nOk\n");
    assert_eq!(Pos{index: 16, x: 0, y: 2}, t.right_rel());
    
    let mut t = Text::new(100);
    text_insert(&mut t, "Hello 😎 ");
    t.backspace_rel();
    t.backspace_rel();
    assert_eq!(Pos{index: 6, x: 6, y: 0}, t.right_rel());
    
    let mut t = Text::new(100);
    t.down_rel();
    assert_eq!(Pos{index: 0, x: 0, y: 0}, t.down_rel());
  }
  
  #[test]
  fn test_insert_at_line_boundary() {
    let mut t = text_init(100, "Hello.\nÉpoustouflant!\nOk.\n");
    assert_eq!(Pos{index: 25, x: 3, y: 2}, t.backspace_rel());
    t.loc = 21;
    assert_eq!(Some(&Line{num: 1, coff: 7, boff: 7, cext: 22,  bext: 23,  chars: 14, bytes: 15, hard: true}), t.line_with_index(t.loc));
    t.insert_rel(' ');
    assert_eq!(22, t.loc);
    assert_eq!("Hello.\nÉpoustouflant! \nOk.", t.text);
    t.insert_rel('Z');
    assert_eq!(23, t.loc);
    assert_eq!("Hello.\nÉpoustouflant! Z\nOk.", t.text);
    t.insert_rel('o');
    assert_eq!(24, t.loc);
    assert_eq!("Hello.\nÉpoustouflant! Zo\nOk.", t.text);
    t.insert_rel('w');
    assert_eq!(25, t.loc);
    assert_eq!("Hello.\nÉpoustouflant! Zow\nOk.", t.text);
    t.insert_rel('.');
    assert_eq!(26, t.loc);
    assert_eq!("Hello.\nÉpoustouflant! Zow.\nOk.", t.text);
  }
  
  #[test]
  fn test_offsets() {
    let t = "A → B"; // '→' is 3 UTF-8 bytes
    let x = Text::new_with_str(100, t);
    assert_eq!(Some(&Line{num: 0, coff: 0, boff: 0, cext: 5, bext: 7, chars: 5, bytes: 7, hard: false}), x.line_with_index(0));
    assert_eq!(Some(&Line{num: 0, coff: 0, boff: 0, cext: 5, bext: 7, chars: 5, bytes: 7, hard: false}), x.line_with_index(1));
    
    let t = "A → B, très bien"; // '→' is 3 UTF-8 bytes, 'è' is 2 UTF-8 bytes
    let x = Text::new_with_str(100, t);
    assert_eq!(Some(&Line{num: 0, coff: 0, boff: 0, cext: 16, bext: 19, chars: 16, bytes: 19, hard: false}), x.line_with_index(9));
    assert_eq!(None, x.line_with_index(16));
    assert_eq!(None, x.line_with_index(99));
    
    let t = "A → B\ntrès bien"; // '→' is 3 UTF-8 bytes, 'è' is 2 UTF-8 bytes
    let x = Text::new_with_str(100, t);
    assert_eq!(Some(&Line{num: 0, coff: 0, boff: 0, cext:  6, bext:  8, chars: 5, bytes:  7, hard: true}), x.line_with_index(1));
    assert_eq!(Some(&Line{num: 1, coff: 6, boff: 8, cext: 15, bext: 18, chars: 9, bytes: 10, hard: false}), x.line_with_index(6));
    
    assert_eq!(Some(1),  x.offset_for_index(1));
    assert_eq!(Some(5),  x.offset_for_index(3));
    assert_eq!(Some(8),  x.offset_for_index(6));
    assert_eq!(Some(12), x.offset_for_index(9));
    assert_eq!(None,     x.offset_for_index(16));
    assert_eq!(None,     x.offset_for_index(99));
    
    // reverse
    assert_eq!(Some(1),  x.index_for_offset(1));
    assert_eq!(Some(3),  x.index_for_offset(5));
    assert_eq!(Some(6),  x.index_for_offset(8));
    assert_eq!(Some(9),  x.index_for_offset(12));
    assert_eq!(None,     x.index_for_offset(19));
    assert_eq!(None,     x.index_for_offset(99));
    
    let t = "Yo!\n";
    let x = Text::new_with_str(100, t);
    assert_eq!(Some(3),  x.offset_for_index(3));
    assert_eq!(None,     x.offset_for_index(4));
  }
  
  #[test]
  fn test_find_fwd() {
    let t = "Très bien, c'est époustouflant !";
    let x = Text::new_with_str(100, t);
    assert_eq!(Some(Pos{index:  4, x:  4, y: 0}), x.find_fwd( 0, match_word_boundary));
    assert_eq!(Some(Pos{index:  4, x:  4, y: 0}), x.find_fwd( 4, match_word_boundary));
    assert_eq!(Some(Pos{index: 30, x: 30, y: 0}), x.find_fwd(24, match_word_boundary));
    assert_eq!(Some(Pos{index: 32, x: 32, y: 0}), x.find_fwd(31, match_word_boundary));

    assert_eq!(Some(Pos{index:  5, x:  5, y: 0}), x.find_fwd( 0, match_word));
    assert_eq!(Some(Pos{index: 11, x: 11, y: 0}), x.find_fwd( 5, match_word));
    assert_eq!(Some(Pos{index: 17, x: 17, y: 0}), x.find_fwd(11, match_word));
    assert_eq!(Some(Pos{index: 31, x: 31, y: 0}), x.find_fwd(17, match_word));
    assert_eq!(Some(Pos{index: 32, x: 32, y: 0}), x.find_fwd(31, match_word));
  }
  
  #[test]
  fn test_find_rev() {
    let t = "Très bien, c'est époustouflant !";
    let x = Text::new_with_str(100, t);
    assert_eq!(Some(Pos{index:  0, x:  0, y: 0}), x.find_rev( 3, match_word_boundary));
    assert_eq!(Some(Pos{index:  5, x:  5, y: 0}), x.find_rev( 9, match_word_boundary));
    assert_eq!(Some(Pos{index:  5, x:  5, y: 0}), x.find_rev( 5, match_word_boundary));
    assert_eq!(Some(Pos{index: 17, x: 17, y: 0}), x.find_rev(24, match_word_boundary));
  }
  
  #[test]
  fn test_iter_lines() {
    let t = "Très bien,\nc'est époustouflant!\nD'acc, à bientôt...";
    let x = Text::new_with_str(100, t);
    let mut it = x.paragraphs();
    assert_eq!(Some(("Très bien,", 1)), it.next());
    assert_eq!(Some(("c'est époustouflant!", 1)), it.next());
    assert_eq!(Some(("D'acc, à bientôt...", 1)), it.next());
    assert_eq!(None, it.next());
    
    let t = "Très bien,\nc'est époustouflant!\nD'acc, à bientôt...";
    let x = Text::new_with_str(5, t);
    let mut it = x.paragraphs();
    assert_eq!(Some(("Très bien,", 3)), it.next());
    assert_eq!(Some(("c'est époustouflant!", 5)), it.next());
    assert_eq!(Some(("D'acc, à bientôt...", 4)), it.next());
    assert_eq!(None, it.next());
  }
  
  #[test]
  fn test_select() {
    let t = "Très bien,\nc'est époustouflant!\nD'acc, à bientôt...";
    let mut x = Text::new_with_str(100, t);
    
    assert_eq!(Pos{index: 10, x: 10, y: 0}, x.select_rel(Some( 0..10), true));
    assert_eq!(Some(0..10), x.selection());
    assert_eq!(Some("Très bien,"), x.selected_text());
    
    assert_eq!(Pos{index: 31, x: 20, y: 1}, x.select_rel(Some(10..31), true));
    assert_eq!(Some(0..31), x.selection());
    assert_eq!(Some("Très bien,\nc'est époustouflant!"), x.selected_text());
    
    assert_eq!(Pos{index: 31, x: 20, y: 1}, x.select_rel(None, true));
    assert_eq!(None, x.selection());
    assert_eq!(None, x.selected_text());
    
    x.to_abs(0);
    
    assert_eq!(Pos{index: 31, x: 20, y: 1}, x.select_rel(Some(10..31), true));
    assert_eq!(Some(10..31), x.selection());
    assert_eq!(Some("\nc'est époustouflant!"), x.selected_text());
    
    assert_eq!(Pos{index: 5, x: 5, y: 0}, x.select_rel(Some(5..10), true));
    assert_eq!(Some(5..31), x.selection());
    assert_eq!(Some("bien,\nc'est époustouflant!"), x.selected_text());
  }
}

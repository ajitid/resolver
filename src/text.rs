use std::fmt;
use std::str;
use std::cmp::min;

use crate::buffer::Buffer;

pub const ZERO_POS: Pos = Pos{x: 0, y: 0, index: 0};

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Pos {
  index: usize,
  pub x: usize,
  pub y: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Line {
  num: usize,
  offset: usize,
  extent: usize,
  chars: usize,
  bytes: usize,
}

impl Line {
  pub fn text<'a>(&self, text: &'a str) -> &'a str {
    &text[self.offset..self.offset + self.bytes]
  }
  
  pub fn extent(&self) -> usize {
    self.extent
  }
  
  pub fn contains(&self, idx: usize) -> bool {
    idx >= self.offset && idx < self.extent
  }
  
  pub fn pos(&self, width: usize, idx: usize) -> Option<Pos> {
    if !self.contains(idx) {
      return None;
    }
    let eix = idx - self.offset;
    if eix < width {
      Some(Pos{index: idx, x: eix, y: self.num})
    }else{
      Some(Pos{index: idx, x: width, y: self.num}) // end of visual line
    }
  }
}

pub struct Text {
  text: String,
  width: usize,
  lines: Vec<Line>,
  loc: usize,
}

impl Text {
  pub fn new(width: usize) -> Text {
    Text{
      text: String::new(),
      width: width,
      lines: Vec::new(),
      loc: 0,
    }
  }
  
  pub fn new_with_str(width: usize, text: &str) -> Text {
    let mut c = Text{
      text: text.to_owned(),
      width: width,
      lines: Vec::new(),
      loc: 0,
    };
    c.reflow();
    c
  }
  
  pub fn new_with_string(width: usize, text: String) -> Text {
    let mut c = Text{
      text: text, // no copy
      width: width,
      lines: Vec::new(),
      loc: 0,
    };
    c.reflow();
    c
  }
  
  pub fn len(&self) -> usize {
    self.text.len()
  }
  
  pub fn width(&self) -> usize {
    self.width
  }
  
  pub fn lines<'a>(&'a self) -> str::Lines<'a> {
    self.text.lines()
  }
  
  pub fn num_lines(&self) -> usize {
    self.lines.len()
  }
  
  pub fn read_line<'a>(&'a self, i: usize) -> Option<&'a str> {
    if i < self.lines.len() {
      Some(self.lines[i].text(&self.text))
    }else{
      None
    }
  }
  
  pub fn write_line(&self, i: usize, b: &mut Buffer) -> usize {
    let t = match self.read_line(i) {
      Some(t) => t,
      None => return 0,
    };
    b.push_str(t);
    t.len()
  }
  
  fn reflow(&mut self) -> &mut Self {
    let mut l: Vec<Line> = Vec::new();
    
    let mut ao: usize = 0; // absolute text offset
    let mut lc: usize = 0; // line width in chars
    let mut lb: usize = 0; // line width in bytes
    let mut lw: usize = 0; // line width to beginning of last whitespace
    let mut lr: usize = 0; // line width to beginning of last non-whitespace
    let mut ly: usize = 0; // line number
    let mut p:  char = '\0'; // previous iteration character
    
    // 0             16
    //             w
    // ┌───────────┐ r
    // ┌─────────────┐
    // Hello this is some text.
    // └──────────────┘
    //                b/c
    
    for c in self.text.chars() {
      if c.is_whitespace() {
        if !p.is_whitespace() { lw = lc; }
      }else{
        if  p.is_whitespace() { lr = lc; }
      }
      
      lb += 1;
      lc += 1;
      
      if Self::is_break(c) || lc >= self.width {
        let br = if lw > 0 { lw } else { lc }; // break
        let cw = if lr > 0 { lr } else { lc }; // consume width
        
        l.push(Line{
          num:    ly,
          offset: ao,
          extent: ao + cw, // abs offset to beginning of break point
          chars:  br,      // width to break point
          bytes:  br,      // same as chars for now
        });
        
        ly += 1;  // increment line number
        ao += cw; // increment absolute offset
        
        let rm = lc - cw; // remaining in the current line to carry over
        lc = rm;
        lb = rm;
        lw = 0; // reset whitespace boundary
        lr = 0; // reset non-whitespace boundary
      }
      
      p = c
    }
    
    if lc > 0 {
      l.push(Line{
        num:    ly,
        offset: ao,
        extent: ao + lb, // abs offset to end of text; last line trails whitespace
        chars:  lc,      // width to end of text; last line trails whitespace
        bytes:  lb,      // same as chars for now
      });
    }
    
    self.lines = l;
    self
  }
  
  fn is_break(c: char) -> bool {
    c == '\n'
  }
  
  pub fn up(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    if pos.y == 0 {
      return ZERO_POS;
    }
    let n = pos.y - 1;
    let line = &self.lines[n];
    if line.chars > pos.x {
      Pos{x: pos.x, y: n, index: line.offset + pos.x}
    }else{
      Pos{x: line.chars, y: n, index: line.extent()}
    }
  }
  
  pub fn up_rel(&mut self) -> Pos {
    let pos = self.up(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn down(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    let n = pos.y + 1;
    if n >= self.lines.len() {
      let line = &self.lines[pos.y];
      return Pos{x: line.chars, y: pos.y, index: line.extent()};
    }
    let line = &self.lines[n];
    if line.chars > pos.x {
      Pos{x: pos.x, y: n, index: line.offset + pos.x}
    }else{
      Pos{x: line.chars, y: n, index: line.extent()}
    }
  }
  
  pub fn down_rel(&mut self) -> Pos {
    let pos = self.down(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn left(&mut self, idx: usize) -> Pos {
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
  
  pub fn right(&mut self, idx: usize) -> Pos {
    self.index(idx + 1)
  }
  
  pub fn right_rel(&mut self) -> Pos {
    let pos = self.right(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn home(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    let line = &self.lines[pos.y];
    Pos{x: 0, y: pos.y, index: line.offset}
  }
  
  pub fn home_rel(&mut self) -> Pos {
    let pos = self.home(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn end(&mut self, idx: usize) -> Pos {
    let pos = self.index(idx);
    let line = &self.lines[pos.y];
    Pos{x: line.chars, y: pos.y, index: line.extent()}
  }
  
  pub fn end_rel(&mut self) -> Pos {
    let pos = self.end(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn index(&mut self, idx: usize) -> Pos {
    return self.index_new(idx); // TESTING
    //
    if idx == 0 {
      return ZERO_POS;
    }
    let idx = min(self.len(), idx);
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut nl: bool = false;
    for line in &self.lines {
      y = line.num;
      if line.contains(idx) {
        let slice = line.text(&self.text);
        let eix = idx - line.offset;
        nl = true;
        for (i, c) in slice.chars().enumerate() {
          if i == eix {
            return Pos{x: i, y: line.num, index: idx};
          }
          x = i;
          nl = c == '\n';
        }
      }
    }
    if nl || x + 1 > self.width {
      Pos{x: 0, y: y+1, index: idx}
    }else{
      Pos{x: x+1, y: y, index: idx}
    }
  }
  
  pub fn index_new(&mut self, idx: usize) -> Pos {
    if idx == 0 {
      return ZERO_POS;
    }
    let idx = min(self.len(), idx);
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut nl: bool = false;
    for line in &self.lines {
      if let Some(pos) = line.pos(self.width, idx) {
        return pos
      }
    }
    // if nl || x + 1 > self.width {
    //   Pos{x: 0, y: y+1, index: idx}
    // }else{
    //   Pos{x: x+1, y: y, index: idx}
    // }
    ZERO_POS
  }
  
  pub fn set_text(&mut self, text: String) {
    self.text = text;
    self.reflow();
  }
  
  pub fn insert(&mut self, idx: usize, c: char) -> Pos {
    self.text.insert(idx, c);
    return self.reflow().index(idx + 1);
  }
  
  pub fn insert_rel(&mut self, c: char) -> Pos {
    let pos = self.insert(self.loc, c);
    self.loc += 1;
    pos
  }
  
  pub fn backspace(&mut self, idx: usize) -> Pos {
    let l = self.text.len();
    if l == 0 { // nothing to delete
      return ZERO_POS;
    }
    self.text.remove(idx);
    return self.reflow().index(idx);
  }
  
  pub fn backspace_rel(&mut self) -> Pos {
    if self.loc == 0 { // nothing to delete
      return ZERO_POS;
    }
    self.loc -= 1;
    self.backspace(self.loc)
  }
}

impl fmt::Display for Text {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let n = self.num_lines();
    for i in 0..n {
      if let Some(l) = self.read_line(i) {
        write!(f, "{}\r\n", l)?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_reflow() {
    let c = Text::new_with_str(100, "Hello");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 5, chars: 5, bytes: 5},
    ], c.lines);
    assert_eq!(vec![
      "Hello"
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
    
    let c = Text::new_with_str(3, "Hello");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 3, chars: 3, bytes: 3},
      Line{num: 1, offset: 3, extent: 5, chars: 2, bytes: 2},
    ], c.lines);
    assert_eq!(vec![
      "Hel",
      "lo",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
    
    let c = Text::new_with_str(8, "Hello there");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6, chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 11, chars: 5, bytes: 5},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
    
    let c = Text::new_with_str(8, "Hello there monchambo");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6, chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 12, chars: 5, bytes: 5},
      Line{num: 2, offset: 12, extent: 20, chars: 8, bytes: 8},
      Line{num: 3, offset: 20, extent: 21, chars: 1, bytes: 1},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there",
      "monchamb",
      "o",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
    
    let c = Text::new_with_str(8, "Hello\nthere monchambo");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6, chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 12, chars: 5, bytes: 5},
      Line{num: 2, offset: 12, extent: 20, chars: 8, bytes: 8},
      Line{num: 3, offset: 20, extent: 21, chars: 1, bytes: 1},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there",
      "monchamb",
      "o",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
    
    let c = Text::new_with_str(100, "Hello\nthere.");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6,  chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 12, chars: 6, bytes: 6},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there.",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());

    let c = Text::new_with_str(100, "Hello\nthere.\n");
    assert_eq!(vec![
      Line{num: 0, offset: 0, extent: 6,  chars: 5, bytes: 5},
      Line{num: 1, offset: 6, extent: 13, chars: 6, bytes: 6},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there.",
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());

    let c = Text::new_with_str(100, "Hello\nthere.\n!");
    assert_eq!(vec![
      Line{num: 0, offset: 0,  extent: 6,  chars: 5, bytes: 5},
      Line{num: 1, offset: 6,  extent: 13, chars: 6, bytes: 6},
      Line{num: 2, offset: 13, extent: 14, chars: 1, bytes: 1},
    ], c.lines);
    assert_eq!(vec![
      "Hello",
      "there.",
      "!"
    ], c.lines.iter().map(|e| { e.text(&c.text) }).collect::<Vec<&str>>());
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
    //
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Text::new_with_str(100, "Hello").index(4));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Hello!\n").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Text::new_with_str(100, "Hello!\n").index(7));
  }
  
  #[test]
  fn test_movement() {
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Hello.").right(5));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Hello.").right(100));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Text::new_with_str(100, "Hello,\nthere").right(6));
    
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Text::new_with_str(100, "Hello.").left(5));
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Text::new_with_str(100, "Hello.").left(0));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Hello,\nthere").left(7));
    
    assert_eq!(Pos{index: 0, x: 0, y: 0}, Text::new_with_str(100, "Hello,\nto\nyourself").up(7));
    assert_eq!(Pos{index: 1, x: 1, y: 0}, Text::new_with_str(100, "Hello,\nto\nyourself").up(8));
    assert_eq!(Pos{index: 9, x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").up(13));
    assert_eq!(Pos{index: 9, x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").up(16));

    assert_eq!(Pos{index: 9, x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").down(2));
    assert_eq!(Pos{index: 9, x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").down(6));
  }
  
}

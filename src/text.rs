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
  num:    usize,
  offset: usize,
  extent: usize,
  chars:  usize,
  bytes:  usize,
  hard:   bool, // does this line break at a literal newline?
}

impl Line {
  pub fn text<'a>(&self, text: &'a str) -> &'a str {
    &text[self.offset..self.offset + self.bytes]
  }
  
  pub fn width(&self) -> usize {
    self.chars
  }
  
  pub fn left(&self) -> usize {
    self.offset
  }
  
  pub fn right(&self) -> usize {
    self.offset + self.bytes
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
  
  fn line_with_index<'a>(&'a self, index: usize) -> Option<(&'a Line, usize)> {
    if self.lines.len() == 0 {
      return None;
    }
    
    let mut co = 0;
    let mut po = 0;
    for (i, l) in self.lines.iter().enumerate() {
      co = po + l.chars;
      if index >= po && index < co {
        return Some((&l, po));
      }
      po = co;
    }
    
    None
  }
  
  fn offset_for_index<'a>(&'a self, index: usize) -> Option<usize> {
    let (line, base) = match self.line_with_index(index) {
      Some((line, base)) => (line, base),
      None => return None,
    };
    
    let mut rem = index - base;
    if rem == 0 {
      return Some(line.offset);
    }
    
    let mut ecw = 0;
    for c in line.text(&self.text).chars() {
      ecw += c.len_utf8();
      rem -= 1;
      if rem == 0 {
        return Some(line.offset + ecw);
      }
    }
    
    None
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
    
    let mut ao: usize = 0; // absolute text offset, in bytes
    let mut lc: usize = 0; // line width, in chars
    let mut lb: usize = 0; // line width, in bytes
    let mut wc: usize = 0; // line width to beginning of last whitespace, in chars
    let mut wb: usize = 0; // line width to beginning of last whitespace, in bytes
    let mut rc: usize = 0; // line width to beginning of last non-whitespace, in chars
    let mut rb: usize = 0; // line width to beginning of last non-whitespace, in bytes
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
      let hard = Self::is_break(c);
      if hard {
        if !p.is_whitespace() {
          rc = lc;
          rb = lb;
        }
        if wc == 0 { // no whitespace boundary; set to here
          wc = lc;
          wb = lb;
        }
      }
      if c.is_whitespace() {
        if !p.is_whitespace() {
          wc = lc;
          wb = lb;
        }
      }else{
        if p.is_whitespace() {
          rc = lc;
          rb = lb;
        }
      }
      
      lc += 1;
      lb += c.len_utf8();
      
      if hard || lc >= self.width {
        let bc = if  hard || wc > 0 { wc } else { lc }; // break
        let bb = if  hard || wb > 0 { wb } else { lb }; // break
        let cc = if !hard && rc > 0 { rc } else { lc }; // consume width, in chars
        let cb = if !hard && rb > 0 { rb } else { lb }; // consume width, in bytes
        
        l.push(Line{
          num:    ly,
          offset: ao,
          extent: ao + cb, // abs offset to beginning of break point, in bytes
          chars:  bc,      // width to break point, in chars
          bytes:  bb,      // width to break point, in bytes
          hard:   hard,    // is this a hard break that ends in a newline literal?
        });
        
        ly += 1;  // increment line number
        ao += cb; // increment absolute offset
        
        lc = lc - cc; // remaining in the current line to carry over, in chars
        lb = lb - cb; // remaining in the current line to carry over, in bytes
        
        wc = 0;   // reset whitespace boundary, in chars
        wb = 0;   // reset whitespace boundary, in bytes
        rc = 0;   // reset non-whitespace boundary, in chars
        rb = 0;   // reset non-whitespace boundary, in bytes
        
        p = '\0';
      }else{
        p = c
      }
    }
    
    if lc > 0 {
      l.push(Line{
        num:    ly,
        offset: ao,
        extent: ao + lb, // abs offset to end of text; last line trails whitespace
        chars:  lc,      // width to end of text, in chars; last line trails whitespace
        bytes:  lb,      // width to end of text, in bytes; last line trails whitespace
        hard:   false,   // can't be a hard break here
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
    let l = &self.lines[n];
    let w = l.width();
    if w > pos.x {
      Pos{x: pos.x, y: n, index: l.offset + pos.x}
    }else{
      Pos{x: w, y: n, index: l.right()}
    }
  }
  
  pub fn up_rel(&mut self) -> Pos {
    let pos = self.up(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn down(&mut self, idx: usize) -> Pos {
    let nl = self.lines.len();
    if nl == 0 {
      return ZERO_POS; // no line data; we have no content
    }
    let pos = self.index(idx);
    let y = if nl > 0 {
      min(nl - 1, pos.y)
    } else {
      0
    };
    let n = y + 1;
    if n >= nl {
      let line = &self.lines[y];
      return Pos{x: line.width(), y: y, index: line.right()};
    }
    let line = &self.lines[n];
    let w = line.width();
    if w > pos.x {
      Pos{x: pos.x, y: n, index: line.offset + pos.x}
    }else{
      Pos{x: w, y: n, index: line.right()}
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
    Pos{x: line.width(), y: pos.y, index: line.right()}
  }
  
  pub fn end_rel(&mut self) -> Pos {
    let pos = self.end(self.loc);
    self.loc = pos.index;
    pos
  }
  
  pub fn index(&mut self, idx: usize) -> Pos {
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
      100, "Hello",
      vec![
        Line{num: 0, offset: 0, extent: 5, chars: 5, bytes: 5, hard: false,},
      ],
      vec![
        "Hello",
      ]
    );
    
    test_reflow_case!(
      3, "Hello",
      vec![
          Line{num: 0, offset: 0, extent: 3, chars: 3, bytes: 3, hard: false},
          Line{num: 1, offset: 3, extent: 5, chars: 2, bytes: 2, hard: false},
      ],
      vec![
        "Hel",
        "lo",
      ]
    );
    
    test_reflow_case!(
      8, "Hello there",
      vec![
        Line{num: 0, offset: 0, extent: 6, chars: 5, bytes: 5, hard: false},
        Line{num: 1, offset: 6, extent: 11, chars: 5, bytes: 5, hard: false},
      ],
      vec![
        "Hello",
        "there",
      ]
    );
    
    test_reflow_case!(
      8, "Hello there monchambo",
      vec![
        Line{num: 0, offset: 0, extent: 6, chars: 5, bytes: 5, hard: false},
        Line{num: 1, offset: 6, extent: 12, chars: 5, bytes: 5, hard: false},
        Line{num: 2, offset: 12, extent: 20, chars: 8, bytes: 8, hard: false},
        Line{num: 3, offset: 20, extent: 21, chars: 1, bytes: 1, hard: false},
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
        Line{num: 0, offset: 0, extent: 6, chars: 5, bytes: 5, hard: true},
        Line{num: 1, offset: 6, extent: 12, chars: 5, bytes: 5, hard: false},
        Line{num: 2, offset: 12, extent: 20, chars: 8, bytes: 8, hard: false},
        Line{num: 3, offset: 20, extent: 21, chars: 1, bytes: 1, hard: false},
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
        Line{num: 0, offset: 0, extent: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, offset: 6, extent: 12, chars: 6, bytes: 6, hard: false},
      ],
      vec![
        "Hello",
        "there.",
      ]
    );

    test_reflow_case!(
      100, "Hello\nthere.\n",
      vec![
        Line{num: 0, offset: 0, extent: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, offset: 6, extent: 13, chars: 6, bytes: 6, hard: true},
      ],
      vec![
        "Hello",
        "there.",
      ]
    );

    test_reflow_case!(
      100, "Hello\nthere.\n!",
      vec![
        Line{num: 0, offset: 0,  extent: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, offset: 6,  extent: 13, chars: 6, bytes: 6, hard: true},
        Line{num: 2, offset: 13, extent: 14, chars: 1, bytes: 1, hard: false},
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
        Line{num: 0, offset: 0,  extent: 6,  chars: 5, bytes: 5, hard: true},
        Line{num: 1, offset: 6,  extent: 14, chars: 7, bytes: 7, hard: true},
        Line{num: 2, offset: 14, extent: 15, chars: 1, bytes: 1, hard: false},
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
        Line{num: 0, offset: 0, extent: 2,  chars: 1, bytes: 1, hard: true},
        Line{num: 1, offset: 2, extent: 4,  chars: 1, bytes: 1, hard: true},
        Line{num: 2, offset: 4, extent: 6,  chars: 1, bytes: 1, hard: true},
        Line{num: 3, offset: 6, extent: 12, chars: 6, bytes: 6, hard: false},
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
        Line{num: 0, offset: 0, extent: 1, chars: 0, bytes: 0, hard: true},
        Line{num: 1, offset: 1, extent: 2, chars: 0, bytes: 0, hard: true},
        Line{num: 2, offset: 2, extent: 3, chars: 0, bytes: 0, hard: true},
        Line{num: 3, offset: 3, extent: 9, chars: 6, bytes: 6, hard: false},
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
        Line{num: 0, offset: 0, extent: 1,  chars: 0, bytes: 0, hard: true},
        Line{num: 1, offset: 1, extent: 8,  chars: 6, bytes: 6, hard: true},
        Line{num: 2, offset: 8, extent: 10, chars: 2, bytes: 2, hard: false},
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
        Line{num: 0, offset: 0, extent: 1,  chars: 0, bytes: 0, hard: true},
        Line{num: 1, offset: 1, extent: 2,  chars: 0, bytes: 0, hard: true},
        Line{num: 2, offset: 2, extent: 7,  chars: 5, bytes: 5, hard: false},
        Line{num: 3, offset: 7, extent: 9,  chars: 1, bytes: 1, hard: true},
        Line{num: 4, offset: 9, extent: 11, chars: 2, bytes: 2, hard: false},
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
    //
    assert_eq!(Pos{index: 4, x: 4, y: 0}, Text::new_with_str(100, "Hello").index(4));
    assert_eq!(Pos{index: 6, x: 6, y: 0}, Text::new_with_str(100, "Hello!\n").index(6));
    assert_eq!(Pos{index: 7, x: 0, y: 1}, Text::new_with_str(100, "Hello!\n").index(7));
  }
  
  #[test]
  fn test_movement() {
    assert_eq!(Pos{index: 6,  x: 6, y: 0}, Text::new_with_str(100, "Hello.").right(5));
    assert_eq!(Pos{index: 6,  x: 6, y: 0}, Text::new_with_str(100, "Hello.").right(100));
    assert_eq!(Pos{index: 7,  x: 0, y: 1}, Text::new_with_str(100, "Hello,\nthere").right(6));
    
    assert_eq!(Pos{index: 4,  x: 4, y: 0}, Text::new_with_str(100, "Hello.").left(5));
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello.").left(0));
    assert_eq!(Pos{index: 6,  x: 6, y: 0}, Text::new_with_str(100, "Hello,\nthere").left(7));
    
    assert_eq!(Pos{index: 0,  x: 0, y: 0}, Text::new_with_str(100, "Hello,\nto\nyourself").up(7));
    assert_eq!(Pos{index: 1,  x: 1, y: 0}, Text::new_with_str(100, "Hello,\nto\nyourself").up(8));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").up(13));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").up(16));

    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").down(2));
    assert_eq!(Pos{index: 9,  x: 2, y: 1}, Text::new_with_str(100, "Hello,\nto\nyourself").down(6));
    assert_eq!(Pos{index: 18, x: 8, y: 2}, Text::new_with_str(100, "Hello,\nto\nyourself").down(18));
  }
  
  #[test]
  fn test_editing() {
    let mut t = Text::new(100);
    t.insert_rel('H');
    t.insert_rel('e');
    t.insert_rel('l');
    t.insert_rel('l');
    t.insert_rel('o');
    t.insert_rel('\n');
    t.insert_rel('O');
    t.insert_rel('k');
    t.insert_rel('\n');
    t.down_rel();
    t.right_rel();
    assert_eq!(Pos{index: 9, x: 0, y: 2}, t.right_rel());
    
    let mut t = Text::new(100);
    t.down_rel();
    assert_eq!(Pos{index: 0, x: 0, y: 0}, t.down_rel());
  }
  
  #[test]
  fn test_offsets() {
    let t = "A → B"; // '→' is 3 UTF-8 bytes
    assert_eq!(Some((&Line{num: 0, offset: 0, extent: 7, chars: 5, bytes: 7, hard: false}, 0)), Text::new_with_str(100, t).line_with_index(0));
    assert_eq!(Some((&Line{num: 0, offset: 0, extent: 7, chars: 5, bytes: 7, hard: false}, 0)), Text::new_with_str(100, t).line_with_index(1));
    
    let t = "A → B, très bien"; // '→' is 3 UTF-8 bytes, 'è' is 2 UTF-8 bytes
    assert_eq!(Some((&Line{num: 0, offset: 0, extent: 19, chars: 16, bytes: 19, hard: false}, 0)), Text::new_with_str(100, t).line_with_index(9));
    
    let t = "A → B\ntrès bien"; // '→' is 3 UTF-8 bytes, 'è' is 2 UTF-8 bytes
    let x = Text::new_with_str(100, t);
    assert_eq!(Some((&Line{num: 0, offset: 0, extent: 8, chars: 5, bytes: 7, hard: true}, 0)), x.line_with_index(1));
    assert_eq!(Some(1),  x.offset_for_index(1));
    assert_eq!(Some(5),  x.offset_for_index(3));
    assert_eq!(Some(8),  x.offset_for_index(5));
    assert_eq!(Some(12), x.offset_for_index(8));
  }
  
}

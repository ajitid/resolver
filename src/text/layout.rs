use crate::text;

fn is_break(c: char) -> bool {
  c == '\n'
}

pub fn layout(text: &str, width: usize) -> Vec<text::Line> {
  let mut l: Vec<text::Line> = Vec::new();
  
  let mut ac: usize = 0; // absolute text offset, in chars
  let mut ab: usize = 0; // absolute text offset, in bytes
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
  
  for c in text.chars() {
    let hard = is_break(c);
    if hard {
      if !p.is_whitespace() {
        rc = lc;
        rb = lb;
      }
      // set whitespace boundary to here
      wc = lc;
      wb = lb;
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
    
    if hard || lc >= width {
      let bc = if  hard || wc > 0 { wc } else { lc }; // break
      let bb = if  hard || wb > 0 { wb } else { lb }; // break
      let cc = if !hard && rc > 0 { rc } else { lc }; // consume width, in chars
      let cb = if !hard && rb > 0 { rb } else { lb }; // consume width, in bytes
      
      l.push(text::Line{
        num:   ly,
        coff:  ac,
        boff:  ab,
        cext:  ac + cc, // abs offset to beginning of break point, in chars
        bext:  ab + cb, // abs offset to beginning of break point, in bytes
        chars: bc,      // width to break point, in chars
        bytes: bb,      // width to break point, in bytes
        hard:  hard,    // is this a hard break that ends in a newline literal?
      });
      
      ly += 1;  // increment line number
      ac += cc; // increment absolute offset, in chars
      ab += cb; // increment absolute offset, in bytes
      
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
    l.push(text::Line{
      num:   ly,
      coff:  ac,
      boff:  ab,
      cext:  ac + lc, // abs offset to end of text, in chars; last line trails whitespace
      bext:  ab + lb, // abs offset to end of text, in bytes; last line trails whitespace
      chars: lc,      // width to end of text, in chars; last line trails whitespace
      bytes: lb,      // width to end of text, in bytes; last line trails whitespace
      hard:  false,   // can't be a hard break here
    });
  }
  
  l
}

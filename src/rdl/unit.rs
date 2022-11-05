use std::fmt;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Unit {
  None(f64),        // indivisible
  
  Teaspoon(f64),    // base
  Tablespoon(f64),  // 3x tsp
  Cup(f64),         // 16x tbsp
  Quart(f64),       // 4x cup
  Gallon(f64),      // 4x quart
  
  Liter(f64),       // base
  Deciliter(f64),   // 1/10 base
  Centiliter(f64),  // 1/100 base
  Milliliter(f64),  // 1/1000 base
  
  Gram(f64),        // base
  Kilogram(f64),    // 1000x grams
}

impl Unit {
  pub fn from(q: f64, n: Option<String>) -> Option<Unit> {
    if let Some(n) = n {
      match n.as_str() {
        "none"    => Some(Self::None(q)),
        
        "tsp"     => Some(Self::Teaspoon(q)),
        "tbsp"    => Some(Self::Tablespoon(q)),
        "cup"     => Some(Self::Cup(q)),
        "quart"   => Some(Self::Quart(q)),
        "gallon"  => Some(Self::Gallon(q)),

        "l"       => Some(Self::Liter(q)),
        "dl"      => Some(Self::Deciliter(q)),
        "cl"      => Some(Self::Centiliter(q)),
        "ml"      => Some(Self::Milliliter(q)),
        
        "g"       => Some(Self::Gram(q)),
        "kg"      => Some(Self::Kilogram(q)),
        _         => None,
      }
    }else{
      Some(Self::None(q))
    }
  }
  
  fn base(&self) -> Unit {
    let mut u = *self;
    loop {
      match u {
        Self::None(n)       => return Self::None(n),

        Self::Teaspoon(n)   => return Self::Teaspoon(n),
        Self::Tablespoon(n) => u = Self::Teaspoon(n * 3.0),
        Self::Cup(n)        => u = Self::Tablespoon(n * 16.0),
        Self::Quart(n)      => u = Self::Cup(n * 4.0),
        Self::Gallon(n)     => u = Self::Quart(n * 4.0),

        Self::Liter(n)      => return Self::Liter(n),
        Self::Deciliter(n)  => u = Self::Liter(n / 10.0),
        Self::Centiliter(n) => u = Self::Deciliter(n / 10.0),
        Self::Milliliter(n) => u = Self::Centiliter(n / 10.0),
        
        Self::Gram(n)       => return Self::Gram(n),
        Self::Kilogram(n)   => u = Self::Gram(n * 1000.0),
      };
    }
  }
  
  fn pack(&self) -> Unit {
    let mut u = *self;
    loop {
      match u {
        Self::None(n) => return Self::None(n),

        Self::Teaspoon(n) => {
          if n < 3.0 {
            return u;
          }else{
            u = Self::Tablespoon(n / 3.0);
          }
        },
        Self::Tablespoon(n) => {
          if n < 4.0 {
            return u;
          }else{
            u = Self::Cup(n / 16.0);
          }
        },
        Self::Cup(n) => {
          if n < 4.0 {
            return u;
          }else{
            u = Self::Quart(n / 4.0);
          }
        },
        Self::Quart(n) => {
          if n < 4.0 {
            return u;
          }else{
            u = Self::Gallon(n / 4.0);
          }
        },
        Self::Gallon(n) => return Self::Gallon(n),

        Self::Milliliter(n) => {
          if n < 10.0 {
            return u;
          }else{
            u = Self::Centiliter(n / 10.0);
          }
        },
        Self::Centiliter(n) => {
          if n < 10.0 {
            return u;
          }else{
            u = Self::Deciliter(n / 10.0);
          }
        },
        Self::Deciliter(n) => {
          if n < 10.0 {
            return u;
          }else{
            u = Self::Liter(n / 10.0);
          }
        },
        Self::Liter(n) => return Self::Liter(n),
        
        Self::Gram(n) => {
          if n < 1000.0 {
            return u;
          }else{
            u = Self::Kilogram(n / 1000.0);
          }
        },
        Self::Kilogram(n) => return Self::Kilogram(n),
      };
    }
  }
}

macro_rules! unit_value_f64 {
  ($variant: path, $value: expr) => {
    if let $variant(v) = $value {
      v as f64
    }else{
      0.0
    }
  };
}

macro_rules! unit_op_f64 {
  ($left: ident, $right: expr, $op: tt) => {
    match $left {
      Self::None(value) => Self::None(value $op unit_value_f64!(Self::None, $right)),
      
      Self::Teaspoon(value) => Self::Teaspoon(value $op unit_value_f64!(Self::Teaspoon, $right)),
      Self::Tablespoon(value) => Self::Tablespoon(value $op unit_value_f64!(Self::Tablespoon, $right)),
      Self::Cup(value) => Self::Cup(value $op unit_value_f64!(Self::Cup, $right)),
      Self::Quart(value) => Self::Quart(value $op unit_value_f64!(Self::Quart, $right)),
      Self::Gallon(value) => Self::Gallon(value $op unit_value_f64!(Self::Gallon, $right)),

      Self::Milliliter(value) => Self::Milliliter(value $op unit_value_f64!(Self::Milliliter, $right)),
      Self::Centiliter(value) => Self::Centiliter(value $op unit_value_f64!(Self::Centiliter, $right)),
      Self::Deciliter(value) => Self::Deciliter(value $op unit_value_f64!(Self::Deciliter, $right)),
      Self::Liter(value) => Self::Liter(value $op unit_value_f64!(Self::Liter, $right)),
      
      Self::Gram(value) => Self::Gram(value $op unit_value_f64!(Self::Gram, $right)),
      Self::Kilogram(value) => Self::Kilogram(value $op unit_value_f64!(Self::Kilogram, $right)),
    }
  };
}

impl ops::Add<Unit> for Unit {
  type Output = Unit;
  
  fn add(self, right: Unit) -> Unit {
    unit_op_f64!(self, right, +)
  }
}

impl ops::Sub<Unit> for Unit {
  type Output = Unit;
  
  fn sub(self, right: Unit) -> Unit {
    unit_op_f64!(self, right, -)
  }
}

impl ops::Mul<Unit> for Unit {
  type Output = Unit;
  
  fn mul(self, right: Unit) -> Unit {
    unit_op_f64!(self, right, *)
  }
}

impl ops::Div<Unit> for Unit {
  type Output = Unit;
  
  fn div(self, right: Unit) -> Unit {
    unit_op_f64!(self, right, /)
  }
}

impl ops::Rem<Unit> for Unit {
  type Output = Unit;
  
  fn rem(self, right: Unit) -> Unit {
    unit_op_f64!(self, right, %)
  }
}

impl fmt::Display for Unit {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.pack() {
      Self::None(n)       => write!(f, "{}", n),
      
      Self::Teaspoon(n)   => write!(f, "{} {}", format_qty(n), "tsp"),
      Self::Tablespoon(n) => write!(f, "{} {}", format_qty(n), "tbsp"),
      Self::Cup(n)        => write!(f, "{} {}", format_qty(n), "cup"),
      Self::Quart(n)      => write!(f, "{} {}", format_qty(n), "quart"),
      Self::Gallon(n)     => write!(f, "{} {}", format_qty(n), "gallon"),
      
      Self::Liter(n)      => write!(f, "{} {}", n, "l"),
      Self::Deciliter(n)  => write!(f, "{} {}", n, "dl"),
      Self::Centiliter(n) => write!(f, "{} {}", n, "cl"),
      Self::Milliliter(n) => write!(f, "{} {}", n, "ml"),
      
      Self::Gram(n)       => write!(f, "{} {}", n, "g"),
      Self::Kilogram(n)   => write!(f, "{} {}", n, "kg"),
    }
  }
}

fn to_fraction(n: f64) -> Option<String> {
  if n == 0.125 {
    Some("1/8".to_string())
  }else if n == 0.25 {
    Some("1/4".to_string())
  }else if n == 0.375 {
    Some("3/8".to_string())
  }else if n == 0.5 {
    Some("1/2".to_string())
  }else if n == 0.625 {
    Some("5/8".to_string())
  }else if n == 0.75 {
    Some("3/4".to_string())
  }else if n == 0.875 {
    Some("7/8".to_string())
  }else{
    None
  }
}

fn format_qty(n: f64) -> String {
  let b = n.floor();
  if let Some(f) = to_fraction(n - b) {
    if b > 0.0 {
      format!("{} {}", b, f)
    }else{
      format!("{}", f)
    }
  }else{
    format!("{}", n)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn to_base() {
    assert_eq!(Unit::Teaspoon(3.0), Unit::Teaspoon(3.0).base());
    
    assert_eq!(Unit::Teaspoon(3.0), Unit::Tablespoon(1.0).base());
    assert_eq!(Unit::Teaspoon(48.0), Unit::Cup(1.0).base());
    assert_eq!(Unit::Teaspoon(192.0), Unit::Quart(1.0).base());
    assert_eq!(Unit::Teaspoon(768.0), Unit::Gallon(1.0).base());
    
    assert_eq!(Unit::Teaspoon(12.0), Unit::Cup(0.25).base());
    assert_eq!(Unit::Teaspoon(24.0), Unit::Cup(0.5).base());
    assert_eq!(Unit::Teaspoon(24.0), Unit::Quart(0.125).base());
    assert_eq!(Unit::Teaspoon(24.0), Unit::Tablespoon(8.0).base());

    assert_eq!(Unit::Liter(0.25), Unit::Liter(0.25).base());
    assert_eq!(Unit::Liter(0.1), Unit::Deciliter(1.0).base());
    assert_eq!(Unit::Liter(0.01), Unit::Centiliter(1.0).base());
    assert_eq!(Unit::Liter(0.001), Unit::Milliliter(1.0).base());
    assert_eq!(Unit::Liter(1.0), Unit::Deciliter(10.0).base());
    assert_eq!(Unit::Liter(1.0), Unit::Centiliter(100.0).base());
    assert_eq!(Unit::Liter(1.0), Unit::Milliliter(1000.0).base());
    assert_eq!(Unit::Liter(3.1), Unit::Milliliter(3100.0).base());

    assert_eq!(Unit::Gram(10.0), Unit::Gram(10.0).base());
    assert_eq!(Unit::Gram(1000.0), Unit::Gram(1000.0).base());
    assert_eq!(Unit::Gram(1000.0), Unit::Kilogram(1.0).base());
    assert_eq!(Unit::Gram(2000.0), Unit::Kilogram(2.0).base());
  }
  
  #[test]
  fn to_pack() {
    assert_eq!(Unit::Teaspoon(2.0), Unit::Teaspoon(2.0).pack());
    assert_eq!(Unit::Tablespoon(1.0), Unit::Teaspoon(3.0).pack());
    assert_eq!(Unit::Cup(0.25), Unit::Teaspoon(12.0).pack());
    assert_eq!(Unit::Cup(1.0), Unit::Teaspoon(48.0).pack());

    assert_eq!(Unit::Tablespoon(3.0), Unit::Tablespoon(3.0).pack());
    assert_eq!(Unit::Tablespoon(3.0), Unit::Tablespoon(3.0).pack());
    assert_eq!(Unit::Cup(0.25), Unit::Tablespoon(4.0).pack());
    assert_eq!(Unit::Cup(1.0), Unit::Tablespoon(16.0).pack());
    assert_eq!(Unit::Cup(3.0), Unit::Tablespoon(48.0).pack());
    assert_eq!(Unit::Quart(1.25), Unit::Tablespoon(80.0).pack());
    assert_eq!(Unit::Quart(3.0), Unit::Tablespoon(192.0).pack());
    assert_eq!(Unit::Gallon(1.25), Unit::Tablespoon(320.0).pack());

    assert_eq!(Unit::Milliliter(1.0), Unit::Milliliter(1.0).pack());
    assert_eq!(Unit::Centiliter(1.0), Unit::Milliliter(10.0).pack());
    assert_eq!(Unit::Deciliter(1.0), Unit::Milliliter(100.0).pack());
    assert_eq!(Unit::Liter(1.0), Unit::Milliliter(1000.0).pack());
    assert_eq!(Unit::Liter(2.1), Unit::Milliliter(2100.0).pack());
    
    assert_eq!(Unit::Gram(999.0), Unit::Gram(999.0).pack());
    assert_eq!(Unit::Kilogram(1.25), Unit::Gram(1250.0).pack());
  }
  
  #[test]
  fn to_display() {
    assert_eq!("1 tsp", Unit::Teaspoon(1.0).to_string());
    assert_eq!("1 1/4 tsp", Unit::Teaspoon(1.25).to_string());
    assert_eq!("2 tsp", Unit::Teaspoon(2.0).to_string());
    
    assert_eq!("1 tbsp", Unit::Teaspoon(3.0).to_string());
    assert_eq!("1/4 cup", Unit::Teaspoon(12.0).to_string());
    assert_eq!("1 cup", Unit::Teaspoon(48.0).to_string());

    assert_eq!("3 tbsp", Unit::Tablespoon(3.0).to_string());
    assert_eq!("1/4 cup", Unit::Tablespoon(4.0).to_string());
    assert_eq!("1/2 cup", Unit::Tablespoon(8.0).to_string());
    assert_eq!("7/8 cup", Unit::Tablespoon(14.0).to_string());
    assert_eq!("2 cup", Unit::Tablespoon(32.0).to_string());
    
    assert_eq!("3 cup", Unit::Cup(3.0).to_string());
    assert_eq!("1 quart", Unit::Cup(4.0).to_string());
    assert_eq!("3 quart", Unit::Cup(12.0).to_string());
    
    assert_eq!("2 1/8 gallon", Unit::Gallon(2.125).to_string());
    assert_eq!("2.123 gallon", Unit::Gallon(2.123).to_string());

    assert_eq!("1 ml", Unit::Milliliter(1.0).to_string());
    assert_eq!("1 cl", Unit::Milliliter(10.0).to_string());
    assert_eq!("1 dl", Unit::Milliliter(100.0).to_string());
    assert_eq!("1 l", Unit::Milliliter(1000.0).to_string());
    assert_eq!("1.1 l", Unit::Milliliter(1100.0).to_string());
    
    assert_eq!("10 g", Unit::Gram(10.0).to_string());
    assert_eq!("2 kg", Unit::Gram(2000.0).to_string());
    assert_eq!("2 kg", Unit::Kilogram(2.0).to_string());
  }
}
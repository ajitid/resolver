use std::fmt;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Unit {
  Teaspoon,    // base
  Tablespoon,  // 3x tsp
  Cup,         // 16x tbsp
  Quart,       // 4x cup
  Gallon,      // 4x quart
  
  Liter,       // base
  Deciliter,   // 1/10 base
  Centiliter,  // 1/100 base
  Milliliter,  // 1/1000 base
  
  Gram,        // base
  Kilogram,    // 1000x grams
}

impl Unit {
  pub fn from(name: &str) -> Option<Unit> {
    match name.to_owned().trim().to_lowercase().as_str() {
      "tsp"            => Some(Unit::Teaspoon),
      "tbsp"           => Some(Unit::Tablespoon),
      "cup" | "cp"     => Some(Unit::Cup),
      "quart" | "qt"   => Some(Unit::Quart),
      "gallon" | "gal" => Some(Unit::Gallon),

      "l"              => Some(Unit::Liter),
      "dl"             => Some(Unit::Deciliter),
      "cl"             => Some(Unit::Centiliter),
      "ml"             => Some(Unit::Milliliter),
      
      "g"              => Some(Unit::Gram),
      "kg"             => Some(Unit::Kilogram),
      _                => None,
    }
  }
  
  pub fn base(&self) -> (Unit, usize) {
    match self {
      Unit::Teaspoon   => (Unit::Teaspoon, 0),
      Unit::Tablespoon => (Unit::Teaspoon, 1),
      Unit::Cup        => (Unit::Teaspoon, 2),
      Unit::Quart      => (Unit::Teaspoon, 3),
      Unit::Gallon     => (Unit::Teaspoon, 4),
      
      Unit::Liter      => (Unit::Liter, 0),
      Unit::Deciliter  => (Unit::Liter, 1),
      Unit::Centiliter => (Unit::Liter, 2),
      Unit::Milliliter => (Unit::Liter, 3),
      
      Unit::Gram       => (Unit::Gram, 0),
      Unit::Kilogram   => (Unit::Gram, 1),
    }
  }
  
  pub fn is_convertable(&self, another: Unit) -> bool {
    let (a, _) = self.base();
    let (b, _) = another.base();
    a == b
  }
}

impl fmt::Display for Unit {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Teaspoon   => write!(f, "{}", "tsp"),
      Self::Tablespoon => write!(f, "{}", "tbsp"),
      Self::Cup        => write!(f, "{}", "cup"),
      Self::Quart      => write!(f, "{}", "quart"),
      Self::Gallon     => write!(f, "{}", "gallon"),
      
      Self::Liter      => write!(f, "{}", "l"),
      Self::Deciliter  => write!(f, "{}", "dl"),
      Self::Centiliter => write!(f, "{}", "cl"),
      Self::Milliliter => write!(f, "{}", "ml"),
      
      Self::Gram       => write!(f, "{}", "g"),
      Self::Kilogram   => write!(f, "{}", "kg"),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Value {
  value: f64,
  unit: Option<Unit>,
}

macro_rules! value_reduce {
  ($curr: ident, $else: expr, $amount: expr, $reduce: expr) => {
    if $curr.value < $amount {
      return $else;
    }else{
      $reduce
    }
  };
}

impl Value {
  pub fn raw(v: f64) -> Value {
    Value{
      value: v,
      unit: None,
    }
  }
  
  pub fn new(v: f64, u: Unit) -> Value {
    Value{
      value: v,
      unit: Some(u),
    }
  }
  
  pub fn option(v: f64, u: Option<Unit>) -> Value {
    Value{
      value: v,
      unit: u,
    }
  }
  
  pub fn value(&self) -> f64 {
    self.value
  }
  
  pub fn unit(&self) -> Option<Unit> {
    self.unit
  }
  
  pub fn is_compatible(&self, with: Option<Unit>) -> bool {
    match self.unit {
      None      => true,
      Some(a)   => match with {
        None    => true,
        Some(b) => a.is_convertable(b),
      }
    }
  }
  
  pub fn untype(&self) -> Value {
    Value{
      value: self.value,
      unit: None,
    }
  }
  
  pub fn convert(&self, unit: Unit) -> Option<Value> {
    let curr = match self.unit {
      Some(curr) => curr,
      None => return Some(Value::new(self.value, unit)),
    };
    
    let (bc, nc) = curr.base();
    let (bt, nt) = unit.base();
    if bc != bt {
      return None;
    }
    
    if nt == nc {
      Some(*self)
    }else if nt > nc {
      self.upcast(unit)
    }else{
      self.downcast(unit)
    }
  }
  
  fn downcast(&self, unit: Unit) -> Option<Value> {
    let mut v = *self;
    loop {
      if v.unit == Some(unit) {
        return Some(v);
      }
      match v.unit {
        None                   => return Some(v),
        
        Some(Unit::Teaspoon)   => return Some(Value::new(v.value, Unit::Teaspoon)),
        Some(Unit::Tablespoon) => v = Value::new(v.value * 3.0, Unit::Teaspoon),
        Some(Unit::Cup)        => v = Value::new(v.value * 16.0, Unit::Tablespoon),
        Some(Unit::Quart)      => v = Value::new(v.value * 4.0, Unit::Cup),
        Some(Unit::Gallon)     => v = Value::new(v.value * 4.0, Unit::Quart),

        Some(Unit::Liter)      => return Some(Value::new(v.value, Unit::Liter)),
        Some(Unit::Deciliter)  => v = Value::new(v.value / 10.0, Unit::Liter),
        Some(Unit::Centiliter) => v = Value::new(v.value / 10.0, Unit::Deciliter),
        Some(Unit::Milliliter) => v = Value::new(v.value / 10.0, Unit::Centiliter),
        
        Some(Unit::Gram)       => return Some(Value::new(v.value, Unit::Gram)),
        Some(Unit::Kilogram)   => v = Value::new(v.value * 1000.0, Unit::Gram),
      };
    }
  }
  
  fn upcast(&self, unit: Unit) -> Option<Value> {
    let mut v = *self;
    loop {
      if v.unit == Some(unit) {
        return Some(v);
      }
      match v.unit {
        None                   => return Some(v),
        
        Some(Unit::Teaspoon)   => v = value_reduce!(v, Some(v), 3.0, Value::new(v.value /  3.0, Unit::Tablespoon)),
        Some(Unit::Tablespoon) => v = value_reduce!(v, Some(v), 4.0, Value::new(v.value / 16.0, Unit::Cup)),
        Some(Unit::Cup)        => v = value_reduce!(v, Some(v), 4.0, Value::new(v.value /  4.0, Unit::Quart)),
        Some(Unit::Quart)      => v = value_reduce!(v, Some(v), 4.0, Value::new(v.value /  4.0, Unit::Gallon)),
        Some(Unit::Gallon)     => return Some(Value::new(v.value, Unit::Gallon)),
        
        Some(Unit::Milliliter) => v = value_reduce!(v, Some(v), 10.0, Value::new(v.value / 10.0, Unit::Centiliter)),
        Some(Unit::Centiliter) => v = value_reduce!(v, Some(v), 10.0, Value::new(v.value / 10.0, Unit::Deciliter)),
        Some(Unit::Deciliter)  => v = value_reduce!(v, Some(v), 10.0, Value::new(v.value / 10.0, Unit::Liter)),
        Some(Unit::Liter)      => return Some(Value::new(v.value, Unit::Liter)),
        
        Some(Unit::Gram)       => v = value_reduce!(v, Some(v), 1000.0, Value::new(v.value / 1000.0, Unit::Kilogram)),
        Some(Unit::Kilogram)   => return Some(Value::new(v.value, Unit::Kilogram)),
      };
    }
  }
  
  fn base(&self) -> Value {
    let mut v = *self;
    loop {
      match v.unit {
        None                   => return Self::raw(v.value),
        
        Some(Unit::Teaspoon)   => return Value::new(v.value, Unit::Teaspoon),
        Some(Unit::Tablespoon) => v = Value::new(v.value * 3.0, Unit::Teaspoon),
        Some(Unit::Cup)        => v = Value::new(v.value * 16.0, Unit::Tablespoon),
        Some(Unit::Quart)      => v = Value::new(v.value * 4.0, Unit::Cup),
        Some(Unit::Gallon)     => v = Value::new(v.value * 4.0, Unit::Quart),

        Some(Unit::Liter)      => return Value::new(v.value, Unit::Liter),
        Some(Unit::Deciliter)  => v = Value::new(v.value / 10.0, Unit::Liter),
        Some(Unit::Centiliter) => v = Value::new(v.value / 10.0, Unit::Deciliter),
        Some(Unit::Milliliter) => v = Value::new(v.value / 10.0, Unit::Centiliter),
        
        Some(Unit::Gram)       => return Value::new(v.value, Unit::Gram),
        Some(Unit::Kilogram)   => v = Value::new(v.value * 1000.0, Unit::Gram),
      };
    }
  }
  
  fn pack(&self) -> Value {
    let mut v = *self;
    loop {
      match v.unit {
        None                   => return Self::raw(v.value),
        
        Some(Unit::Teaspoon)   => v = value_reduce!(v, v, 3.0, Value::new(v.value /  3.0, Unit::Tablespoon)),
        Some(Unit::Tablespoon) => v = value_reduce!(v, v, 4.0, Value::new(v.value / 16.0, Unit::Cup)),
        Some(Unit::Cup)        => v = value_reduce!(v, v, 4.0, Value::new(v.value /  4.0, Unit::Quart)),
        Some(Unit::Quart)      => v = value_reduce!(v, v, 4.0, Value::new(v.value /  4.0, Unit::Gallon)),
        Some(Unit::Gallon)     => return Value::new(v.value, Unit::Gallon),
        
        Some(Unit::Milliliter) => v = value_reduce!(v, v, 10.0, Value::new(v.value / 10.0, Unit::Centiliter)),
        Some(Unit::Centiliter) => v = value_reduce!(v, v, 10.0, Value::new(v.value / 10.0, Unit::Deciliter)),
        Some(Unit::Deciliter)  => v = value_reduce!(v, v, 10.0, Value::new(v.value / 10.0, Unit::Liter)),
        Some(Unit::Liter)      => return Value::new(v.value, Unit::Liter),
        
        Some(Unit::Gram)       => v = value_reduce!(v, v, 1000.0, Value::new(v.value / 1000.0, Unit::Kilogram)),
        Some(Unit::Kilogram)   => return Value::new(v.value, Unit::Kilogram),
      };
    }
  }
}

fn coalesce<T>(a: Option<T>, b: Option<T>) -> Option<T> {
  if let Some(a) = a {
    Some(a)
  }else if let Some(b) = b {
    Some(b)
  }else{
    None
  }
}

impl ops::Add<Value> for Value {
  type Output = Value;
  
  fn add(self, right: Value) -> Value {
    Value{
      value: self.value + right.value,
      unit: coalesce(self.unit, right.unit),
    }
  }
}

impl ops::Sub<Value> for Value {
  type Output = Value;
  
  fn sub(self, right: Value) -> Value {
    Value{
      value: self.value - right.value,
      unit: coalesce(self.unit, right.unit),
    }
  }
}

impl ops::Mul<Value> for Value {
  type Output = Value;
  
  fn mul(self, right: Value) -> Value {
    Value{
      value: self.value * right.value,
      unit: coalesce(self.unit, right.unit),
    }
  }
}

impl ops::Div<Value> for Value {
  type Output = Value;
  
  fn div(self, right: Value) -> Value {
    Value{
      value: self.value / right.value,
      unit: coalesce(self.unit, right.unit),
    }
  }
}

impl ops::Rem<Value> for Value {
  type Output = Value;
  
  fn rem(self, right: Value) -> Value {
    Value{
      value: self.value % right.value,
      unit: coalesce(self.unit, right.unit),
    }
  }
}

impl fmt::Display for Value {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.alternate() {
      match self.unit {
        Some(unit) => write!(f, "{} {}", format_qty(self.value), unit),
        None       => write!(f, "{}", format_qty(self.value)),
      }
    }else{
      match self.unit {
        Some(unit) => write!(f, "{} {}", self.value, unit),
        None       => write!(f, "{}", self.value),
      }
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
    assert_eq!(Value::new(3.0, Unit::Teaspoon), Value::new(3.0, Unit::Teaspoon).base());
    
    assert_eq!(Value::new(3.0, Unit::Teaspoon), Value::new(1.0, Unit::Tablespoon).base());
    assert_eq!(Value::new(48.0, Unit::Teaspoon), Value::new(1.0, Unit::Cup).base());
    assert_eq!(Value::new(192.0, Unit::Teaspoon), Value::new(1.0, Unit::Quart).base());
    assert_eq!(Value::new(768.0, Unit::Teaspoon), Value::new(1.0, Unit::Gallon).base());
    
    assert_eq!(Value::new(12.0, Unit::Teaspoon), Value::new(0.25, Unit::Cup).base());
    assert_eq!(Value::new(24.0, Unit::Teaspoon), Value::new(0.5, Unit::Cup).base());
    assert_eq!(Value::new(24.0, Unit::Teaspoon), Value::new(0.125, Unit::Quart).base());
    assert_eq!(Value::new(24.0, Unit::Teaspoon), Value::new(8.0, Unit::Tablespoon).base());

    assert_eq!(Value::new(0.25, Unit::Liter), Value::new(0.25, Unit::Liter).base());
    assert_eq!(Value::new(0.1, Unit::Liter), Value::new(1.0, Unit::Deciliter).base());
    assert_eq!(Value::new(0.01, Unit::Liter), Value::new(1.0, Unit::Centiliter).base());
    assert_eq!(Value::new(0.001, Unit::Liter), Value::new(1.0, Unit::Milliliter).base());
    assert_eq!(Value::new(1.0, Unit::Liter), Value::new(10.0, Unit::Deciliter).base());
    assert_eq!(Value::new(1.0, Unit::Liter), Value::new(100.0, Unit::Centiliter).base());
    assert_eq!(Value::new(1.0, Unit::Liter), Value::new(1000.0, Unit::Milliliter).base());
    assert_eq!(Value::new(3.1, Unit::Liter), Value::new(3100.0, Unit::Milliliter).base());

    assert_eq!(Value::new(10.0, Unit::Gram), Value::new(10.0, Unit::Gram).base());
    assert_eq!(Value::new(1000.0, Unit::Gram), Value::new(1000.0, Unit::Gram).base());
    assert_eq!(Value::new(1000.0, Unit::Gram), Value::new(1.0, Unit::Kilogram).base());
    assert_eq!(Value::new(2000.0, Unit::Gram), Value::new(2.0, Unit::Kilogram).base());
  }
  
  #[test]
  fn to_pack() {
    assert_eq!(Value::new(2.0, Unit::Teaspoon), Value::new(2.0, Unit::Teaspoon).pack());
    assert_eq!(Value::new(1.0, Unit::Tablespoon), Value::new(3.0, Unit::Teaspoon).pack());
    assert_eq!(Value::new(0.25, Unit::Cup), Value::new(12.0, Unit::Teaspoon).pack());
    assert_eq!(Value::new(1.0, Unit::Cup), Value::new(48.0, Unit::Teaspoon).pack());

    assert_eq!(Value::new(3.0, Unit::Tablespoon), Value::new(3.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(3.0, Unit::Tablespoon), Value::new(3.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(0.25, Unit::Cup), Value::new(4.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(1.0, Unit::Cup), Value::new(16.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(3.0, Unit::Cup), Value::new(48.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(1.25, Unit::Quart), Value::new(80.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(3.0, Unit::Quart), Value::new(192.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(1.25, Unit::Gallon), Value::new(320.0, Unit::Tablespoon).pack());

    assert_eq!(Value::new(1.0, Unit::Milliliter), Value::new(1.0, Unit::Milliliter).pack());
    assert_eq!(Value::new(1.0, Unit::Centiliter), Value::new(10.0, Unit::Milliliter).pack());
    assert_eq!(Value::new(1.0, Unit::Deciliter), Value::new(100.0, Unit::Milliliter).pack());
    assert_eq!(Value::new(1.0, Unit::Liter), Value::new(1000.0, Unit::Milliliter).pack());
    assert_eq!(Value::new(2.1, Unit::Liter), Value::new(2100.0, Unit::Milliliter).pack());
    
    assert_eq!(Value::new(999.0, Unit::Gram), Value::new(999.0, Unit::Gram).pack());
    assert_eq!(Value::new(1.25, Unit::Kilogram), Value::new(1250.0, Unit::Gram).pack());
  }
  
  #[test]
  fn to_display() {
    assert_eq!("1 tsp", &format!("{:#}", Value::new(1.0, Unit::Teaspoon).pack()));
    assert_eq!("1 1/4 tsp", &format!("{:#}", Value::new(1.25, Unit::Teaspoon).pack()));
    assert_eq!("2 tsp", &format!("{:#}", Value::new(2.0, Unit::Teaspoon).pack()));
    
    assert_eq!("1 tbsp", &format!("{:#}", Value::new(3.0, Unit::Teaspoon).pack()));
    assert_eq!("1/4 cup", &format!("{:#}", Value::new(12.0, Unit::Teaspoon).pack()));
    assert_eq!("1 cup", &format!("{:#}", Value::new(48.0, Unit::Teaspoon).pack()));

    assert_eq!("3 tbsp", &format!("{:#}", Value::new(3.0, Unit::Tablespoon).pack()));
    assert_eq!("1/4 cup", &format!("{:#}", Value::new(4.0, Unit::Tablespoon).pack()));
    assert_eq!("1/2 cup", &format!("{:#}", Value::new(8.0, Unit::Tablespoon).pack()));
    assert_eq!("7/8 cup", &format!("{:#}", Value::new(14.0, Unit::Tablespoon).pack()));
    assert_eq!("2 cup", &format!("{:#}", Value::new(32.0, Unit::Tablespoon).pack()));
    
    assert_eq!("3 cup", &format!("{:#}", Value::new(3.0, Unit::Cup).pack()));
    assert_eq!("1 quart", &format!("{:#}", Value::new(4.0, Unit::Cup).pack()));
    assert_eq!("3 quart", &format!("{:#}", Value::new(12.0, Unit::Cup).pack()));
    
    assert_eq!("2 1/8 gallon", &format!("{:#}", Value::new(2.125, Unit::Gallon).pack()));
    assert_eq!("2.123 gallon", &format!("{:#}", Value::new(2.123, Unit::Gallon).pack()));

    assert_eq!("1 ml", &format!("{:#}", Value::new(1.0, Unit::Milliliter).pack()));
    assert_eq!("1 cl", &format!("{:#}", Value::new(10.0, Unit::Milliliter).pack()));
    assert_eq!("1 dl", &format!("{:#}", Value::new(100.0, Unit::Milliliter).pack()));
    assert_eq!("1 l", &format!("{:#}", Value::new(1000.0, Unit::Milliliter).pack()));
    assert_eq!("1.1 l", &format!("{:#}", Value::new(1100.0, Unit::Milliliter).pack()));
    
    assert_eq!("10 g", &format!("{:#}", Value::new(10.0, Unit::Gram).pack()));
    assert_eq!("2 kg", &format!("{:#}", Value::new(2000.0, Unit::Gram).pack()));
    assert_eq!("2 kg", &format!("{:#}", Value::new(2.0, Unit::Kilogram).pack()));
  }

  #[test]
  fn casts() {
    assert_eq!(Some(Value::new(5.0, Unit::Teaspoon)), Value::raw(5.0).convert(Unit::Teaspoon));
    assert_eq!(Some(Value::new(15.0, Unit::Teaspoon)), Value::new(5.0, Unit::Tablespoon).convert(Unit::Teaspoon));
    assert_eq!(Some(Value::new(1.0, Unit::Cup)), Value::new(16.0, Unit::Tablespoon).convert(Unit::Cup));
    assert_eq!(None, Value::new(16.0, Unit::Tablespoon).convert(Unit::Liter));
  }
  
  #[test]
  fn operations() {
    assert_eq!(Value::raw(10.0), Value::raw(5.0) * Value::raw(2.0));
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::new(5.0, Unit::Teaspoon) * Value::new(2.0, Unit::Teaspoon));
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::new(5.0, Unit::Teaspoon) * Value::raw(2.0));
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::raw(2.0) * Value::new(5.0, Unit::Teaspoon));
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::new(5.0, Unit::Teaspoon) * Value::new(2.0, Unit::Tablespoon));
  }
}
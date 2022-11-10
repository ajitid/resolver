use std::fmt;
use std::ops;

const CONVERSION: [[f64; 11]; 11] = [
 //                 Teaspoon,     Tablespoon,         Cup,                 Quart,               Gallon,              Liter,               Deciliter,           Centiliter,        Milliliter,        Gram,      Kilogram,
 /* Teaspoon */   [ 1.0,          1.0 / 3.0,          0.0208333333333333,  0.0052083333333333,  0.0013020833333333,  0.0049289249029002,  0.0492892490290018,  4.92892490290018,  4928.92490290018,  0.0,       0.0 ],
 /* Tablespoon */ [ 3.0,          1.0,                0.0625,              0.015625,            0.00390625,          0.0147867747087005,  0.147867747087005,   14.7867747087005,  14786.7747087005,  0.0,       0.0 ],
 /* Cup */        [ 48.0,         16.0,               1.0,                 0.25,                0.0625,              0.236588395339209,   1.47867747087005,    1478.67747087005,  14786774.7087005,  0.0,       0.0 ],
 /* Quart */      [ 192.0,        64.0,               4.0,                 1.0,                 0.25,                0.946353581356835,   9.46353581356834,    946.353581356834,  946353.581356834,  0.0,       0.0 ],
 /* Gallon */     [ 768.0,        256.0,              16.0,                4.0,                 1.0,                 3.78541432542734,    37.8541432542734,    3785.41432542734,  3785414.32542734,  0.0,       0.0 ],
 /* Liter */      [ 202.884,      67.628,             4.22675,             1.0566875,           0.264171875,         1.0,                 10.0,                100.0,             1000.0,            0.0,       0.0 ],
 /* Deciliter */  [ 20.2884,      6.7628,             0.67628,             0.10566875,          0.0264171875,        0.1,                 1.0,                 10.0,              100.0,             0.0,       0.0 ],
 /* Centiliter */ [ 0.202884,     0.067628,           0.00067628,          0.0010566875,        0.000264171875,      0.01,                0.1,                 1.0,               10.0,              0.0,       0.0 ],
 /* Milliliter */ [ 0.000202884,  0.000067628,        0.000000067628,      0.0000010566875,     0.000000264171875,   0.001,               0.01,                0.1,               1.0,               0.0,       0.0 ],
 /* Gram */       [ 0.0,          0.0,                0.0,                 0.0,                 0.0,                 0.0,                 0.0,                 0.0,               0.0,               1.0,       0.001 ],
 /* Kilogram */   [ 0.0,          0.0,                0.0,                 0.0,                 0.0,                 0.0,                 0.0,                 0.0,               0.0,               1000.0,    1.0 ],
];

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
      "tsp" | "tsps"       => Some(Unit::Teaspoon),
      "tbsp" | "tbsps"     => Some(Unit::Tablespoon),
      "cup" | "cups"       => Some(Unit::Cup),
      "quart" | "quarts"   => Some(Unit::Quart),
      "gallon" | "gallons" => Some(Unit::Gallon),
      
      "l"                  => Some(Unit::Liter),
      "dl"                 => Some(Unit::Deciliter),
      "cl"                 => Some(Unit::Centiliter),
      "ml"                 => Some(Unit::Milliliter),
      
      "g"                  => Some(Unit::Gram),
      "kg"                 => Some(Unit::Kilogram),
      
      _                    => None,
    }
  }
  
  pub fn ordinal(&self) -> usize {
    match self {
      Unit::Teaspoon   => 0,
      Unit::Tablespoon => 1,
      Unit::Cup        => 2,
      Unit::Quart      => 3,
      Unit::Gallon     => 4,
      
      Unit::Liter      => 5,
      Unit::Deciliter  => 6,
      Unit::Centiliter => 7,
      Unit::Milliliter => 8,
      
      Unit::Gram       => 9,
      Unit::Kilogram   => 10,
    }
  }
  
  pub fn up(&self) -> Option<Unit> {
    match self {
      Unit::Teaspoon   => Some(Unit::Tablespoon),
      Unit::Tablespoon => Some(Unit::Cup),
      Unit::Cup        => Some(Unit::Quart),
      Unit::Quart      => Some(Unit::Gallon),
      Unit::Gallon     => None,
      
      Unit::Milliliter => Some(Unit::Centiliter),
      Unit::Centiliter => Some(Unit::Deciliter),
      Unit::Deciliter  => Some(Unit::Liter),
      Unit::Liter      => None,
      
      Unit::Gram       => Some(Unit::Kilogram),
      Unit::Kilogram   => None,
    }
  }
  
  pub fn min(&self) -> Unit {
    match self {
      Unit::Teaspoon   => Unit::Teaspoon,
      Unit::Tablespoon => Unit::Teaspoon,
      Unit::Cup        => Unit::Teaspoon,
      Unit::Quart      => Unit::Teaspoon,
      Unit::Gallon     => Unit::Teaspoon,
      
      Unit::Liter      => Unit::Liter,
      Unit::Deciliter  => Unit::Liter,
      Unit::Centiliter => Unit::Liter,
      Unit::Milliliter => Unit::Liter,
      
      Unit::Gram       => Unit::Gram,
      Unit::Kilogram   => Unit::Gram,
    }
  }
  
  pub fn max(&self) -> Unit {
    match self {
      Unit::Teaspoon   => Unit::Gallon,
      Unit::Tablespoon => Unit::Gallon,
      Unit::Cup        => Unit::Gallon,
      Unit::Quart      => Unit::Gallon,
      Unit::Gallon     => Unit::Gallon,
      
      Unit::Liter      => Unit::Liter,
      Unit::Deciliter  => Unit::Liter,
      Unit::Centiliter => Unit::Liter,
      Unit::Milliliter => Unit::Liter,
      
      Unit::Gram       => Unit::Kilogram,
      Unit::Kilogram   => Unit::Kilogram,
    }
  }
  
  pub fn is_convertable(&self, to: Unit) -> bool {
    CONVERSION[self.ordinal()][to.ordinal()] != 0.0
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
  
  pub fn untype(&self) -> Value {
    Value{
      value: self.value,
      unit: None,
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
  
  pub fn convert(&self, to: Option<Unit>) -> Option<Value> {
    let to = match to {
      Some(to) => to,
      None => return Some(Value::raw(self.value)),
    };
    let from = match self.unit {
      Some(from) => from,
      None => return Some(Value::new(self.value, to)),
    };
    if from == to {
      return Some(*self);
    }
    let factor = CONVERSION[from.ordinal()][to.ordinal()];
    if factor == 0.0 {
      None // cannot convert
    }else{
      Some(Value::new(self.value * factor, to))
    }
  }
  
  fn base(&self) -> Value {
    match self.unit {
      None       => *self,
      Some(unit) => self.convert(Some(unit.min())).unwrap(),
    }
  }
  
  fn pack(&self) -> Value {
    let unit = match self.unit {
      Some(unit) => unit,
      None => return *self,
    };
    
    let mut curr = unit.ordinal();
    let mut v = *self;
    loop {
      let c = match v.unit {
        Some(c) => c,
        None => return v,
      };
      let n = match c.up() {
        Some(n) => v.convert(Some(n)),
        None => return v,
      };
      v = match n {
        None => return v,
        Some(n) => if n.value < 1.0 {
          return v;
        } else {
          n
        },
      }
    }
    
    v // just use the remainder
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

fn operands(left: Value, right: Value) -> (Option<Unit>, Value, Value) {
  let target = coalesce(right.unit, left.unit);
  let left = match left.convert(target) {
    Some(conv) => conv,
    None => left.untype(),
  };
  let right = match right.convert(target) {
    Some(conv) => conv,
    None => right.untype(),
  };
  (target, left, right)
}

impl ops::Add<Value> for Value {
  type Output = Value;
  
  fn add(self, right: Value) -> Value {
    let (target, left, right) = operands(self, right);
    Value{
      value: left.value + right.value,
      unit: target,
    }
  }
}

impl ops::Sub<Value> for Value {
  type Output = Value;
  
  fn sub(self, right: Value) -> Value {
    let (target, left, right) = operands(self, right);
    Value{
      value: left.value - right.value,
      unit: target,
    }
  }
}

impl ops::Mul<Value> for Value {
  type Output = Value;
  
  fn mul(self, right: Value) -> Value {
    let (target, left, right) = operands(self, right);
    Value{
      value: left.value * right.value,
      unit: target,
    }
  }
}

impl ops::Div<Value> for Value {
  type Output = Value;
  
  fn div(self, right: Value) -> Value {
    let (target, left, right) = operands(self, right);
    Value{
      value: left.value / right.value,
      unit: target,
    }
  }
}

impl ops::Rem<Value> for Value {
  type Output = Value;
  
  fn rem(self, right: Value) -> Value {
    let (target, left, right) = operands(self, right);
    Value{
      value: left.value % right.value,
      unit: target,
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
    assert_eq!(Value::new(4.0, Unit::Tablespoon), Value::new(12.0, Unit::Teaspoon).pack());
    assert_eq!(Value::new(1.0, Unit::Cup), Value::new(48.0, Unit::Teaspoon).pack());

    assert_eq!(Value::new(3.0, Unit::Tablespoon), Value::new(3.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(3.0, Unit::Tablespoon), Value::new(3.0, Unit::Tablespoon).pack());
    assert_eq!(Value::new(4.0, Unit::Tablespoon), Value::new(4.0, Unit::Tablespoon).pack());
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
    assert_eq!("4 tbsp", &format!("{:#}", Value::new(12.0, Unit::Teaspoon).pack()));
    assert_eq!("1 cup", &format!("{:#}", Value::new(48.0, Unit::Teaspoon).pack()));

    assert_eq!("3 tbsp", &format!("{:#}", Value::new(3.0, Unit::Tablespoon).pack()));
    assert_eq!("4 tbsp", &format!("{:#}", Value::new(4.0, Unit::Tablespoon).pack()));
    assert_eq!("8 tbsp", &format!("{:#}", Value::new(8.0, Unit::Tablespoon).pack()));
    assert_eq!("14 tbsp", &format!("{:#}", Value::new(14.0, Unit::Tablespoon).pack()));
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
  fn convert() {
    assert_eq!(Some(Value::raw(1.0)), Value::new(1.0, Unit::Tablespoon).convert(None));
    
    assert_eq!(Some(Value::new(3.0, Unit::Teaspoon)), Value::new(1.0, Unit::Tablespoon).convert(Some(Unit::Teaspoon)));
    assert_eq!(Some(Value::new(1.0, Unit::Tablespoon)), Value::new(3.0, Unit::Teaspoon).convert(Some(Unit::Tablespoon)));
    
    assert_eq!(Some(Value::new(5.0, Unit::Teaspoon)), Value::raw(5.0).convert(Some(Unit::Teaspoon)));
    assert_eq!(Some(Value::new(15.0, Unit::Teaspoon)), Value::new(5.0, Unit::Tablespoon).convert(Some(Unit::Teaspoon)));
    assert_eq!(Some(Value::new(1.0, Unit::Cup)), Value::new(16.0, Unit::Tablespoon).convert(Some(Unit::Cup)));
    assert_eq!(Some(Value::new(0.236588395339208, Unit::Liter)), Value::new(16.0, Unit::Tablespoon).convert(Some(Unit::Liter)));
  }
  
  #[test]
  fn operations() {
    assert_eq!(Value::raw(10.0), Value::raw(5.0) * Value::raw(2.0));
    
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::new(5.0, Unit::Teaspoon) * Value::new(2.0, Unit::Teaspoon));
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::new(5.0, Unit::Teaspoon) * Value::raw(2.0));
    assert_eq!(Value::new(10.0, Unit::Teaspoon), Value::raw(2.0) * Value::new(5.0, Unit::Teaspoon));
    assert_eq!(Value::new(20.0, Unit::Tablespoon), Value::new(30.0, Unit::Teaspoon) * Value::new(2.0, Unit::Tablespoon));
  }
}
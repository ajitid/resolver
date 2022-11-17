
pub fn coalesce<T>(a: Option<T>, b: Option<T>) -> Option<T> {
  if let Some(a) = a {
    Some(a)
  }else if let Some(b) = b {
    Some(b)
  }else{
    None
  }
}

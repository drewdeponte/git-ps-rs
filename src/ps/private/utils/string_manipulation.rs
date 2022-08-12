pub fn strip_newlines(s: &str) -> String {
  return s.replace('\n', "").replace('\r', "");
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_trim_newlines() {
    let text = "hel
l

o
";
    assert_eq!(super::strip_newlines(text), "hello");
  }
}
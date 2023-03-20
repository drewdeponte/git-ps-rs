pub fn strip_newlines(s: &str) -> String {
    return s.replace('\n', "").replace('\r', "");
}

pub fn set_string_width(s: &str, width: usize) -> String {
    let trimmed_str = format!("{:.len$}", s, len = width);
    return format!("{:<len$}", trimmed_str, len = width);
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

    #[test]
    fn test_set_string_width_longer() {
        let text = "hello";
        assert_eq!(super::set_string_width(text, 10), "hello     ")
    }

    #[test]
    fn test_set_string_width_shorter() {
        let text = "hello";
        assert_eq!(super::set_string_width(text, 3), "hel")
    }
}

pub fn dot_trim(mut s: String, n: usize) -> String {
    if let Some(dot_pos) = s.find('.') {
        s.remove(dot_pos);
        // let keep_len = (dot_pos + n).min(s.len());
        let keep_len = dot_pos + n;
        if keep_len <= s.len() {
            s.truncate(keep_len);
        } else {
            s.push_str(&"0".repeat(keep_len - s.len()));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_padding() {
        assert_eq!(dot_trim(String::from("0.0358"), 5), "003580");
    }
}
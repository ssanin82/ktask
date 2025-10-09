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

pub fn itos(v: i32, prec: usize) -> String {
    let mut sv = v.to_string();
    if prec < sv.len() {
        sv.insert(sv.len() - prec, '.');
    } else if prec == sv.len() {
        sv.insert_str(0, &"0.");
    } else {
        sv.insert_str(0, &"0".repeat(prec - sv.len()));
        sv.insert_str(0, &"0.");
    }
    sv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teat_dot_trim() {
        assert_eq!(dot_trim(String::from("0.0358"), 5), "003580"); // no padding
        assert_eq!(dot_trim(String::from("12.345"), 3), "12345");
        assert_eq!(dot_trim(String::from("12.345"), 1), "123");
        assert_eq!(dot_trim(String::from("12.345"), 10), "123450000000");
        // some real values
        assert_eq!(dot_trim(String::from("19.46930000"), 6), "19469300");
        assert_eq!(dot_trim(String::from("1.26200000"), 6), "1262000");
    }

    #[test]
    fn teat_itos() {
        assert_eq!(itos(358, 2), "3.58");
        assert_eq!(itos(358, 3), "0.358");
        assert_eq!(itos(358, 5), "0.00358");
    }
}
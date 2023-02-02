pub fn unicode_emojis_num(content: &str) -> i64 {
    let mut num: i64 = 0;
    for c in content.chars() {
        let ord = c as u32;
        if ord >= 127744 && ord <= 129750 {
            num += 1;
        } else if ord >= 126980 && ord <= 127569 {
            num += 1;
        } else if ord >= 169 && ord <= 174 {
            num += 1;
        } else if ord >= 8205 && ord <= 12953 {
            num += 1;
        }
    }
    num
}

pub fn check_obnoxious_unicode(c: char) -> bool {
    let ord = c as u32;
    
    if ord >= 0x20dd && ord <= 0x218f {
        return false
    }

    //if ord >= 0x2200 && ord <= 0x22ff { // Mathematical Operators
    //    return false
    //}

    if (ord >= 0x2300 && ord <= 0x2319) || (ord > 0x231c && ord < 0x23cd) || (ord > 0x23d0 && ord < 0x23e8) { // Miscellaneous Technical minus Emojis
        return false
    }

    if ord >= 0x2400 && ord <= 0x243f { // Control Pictures
        return false
    }

    if ord >= 0x2460 && ord <= 0x24ff && ord != 0x24c2 { // Enclosed Alphanumerics minus Emoji
        return false
    }
    if ord >= 0x1d400 && ord <= 0x1d7ff { // Mathematical Alphanunmerics (a lot of annoying fonts)
        return false
    }
    return true
}

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
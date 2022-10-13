use lazy_static::lazy_static;
use regex::*;

//pub fn clean(s: String) -> String {
//    let mut s = s.clone();
//    s.retain(|c| !c.is_control());
//    s
//}

pub fn remove_spaces(s: &String) -> String {
    let mut s = s.clone();
    s.retain(|c| !c.is_whitespace());
    s
}

pub fn replace_non_std_space(s: &String) -> String {
    let mut s = s.clone();

    lazy_static! {
        static ref NON_STD_RE: Regex = Regex::new(r"[\x{2000}-\x{200F}]+").unwrap();
    }
    s = NON_STD_RE.replace_all(&s, "").to_string();

    s
}
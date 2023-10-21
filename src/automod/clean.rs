use crate::util;

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

    s = util::regex::NON_STD_SP.replace_all(&s, "").to_string();

    s
}

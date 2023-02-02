pub fn mentions_from_id_str_vec(ids: &Vec<String>) -> String {
    let mut mentions = String::new();
    for id in ids {
        mentions.push_str(&format!("<@{}>", id));
    }
    mentions
}

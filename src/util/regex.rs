use regex::Regex;

lazy_static::lazy_static! {
    pub static ref UUID: Regex =
        Regex::new(r"\b[0-9a-f]{8}\b-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-\b[0-9a-f]{12}\b")
        .unwrap();

    pub static ref DOMAINS: Regex =
        Regex::new(r"[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}")
        .unwrap();

    pub static ref IP: Regex =
        Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}")
        .unwrap();

    pub static ref NON_STD_SP: Regex =
        Regex::new(r"[\x{2000}-\x{200F}]+")
        .unwrap();

    pub static ref EMOJI: Regex =
        Regex::new(r"<a?:([a-zA-Z0-9_]+):[0-9]{17,19}>")
        .unwrap();

    pub static ref DURATION: Regex =
        Regex::new(r"(\d+)\S*(y|mo|w|d|h|m|s)")
        .unwrap();

    pub static ref SNOWFLAKE: Regex =
        Regex::new(r"([0-9]{17,19})")
        .unwrap();
}

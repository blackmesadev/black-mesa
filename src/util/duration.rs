use crate::util;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Duration {
    pub years: i64,
    pub months: i64,
    pub weeks: i64,
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub full_string: String,
}

impl Duration {
    pub fn new(str: String) -> Self {
        let mut dur = Self::default();

        for cap in util::regex::DURATION.captures_iter(&str) {
            dur.full_string.push_str(&cap[0]);
            let num = cap[1].parse::<i64>().unwrap();
            let unit = &cap[2];

            match unit {
                "y" => dur.years = num,
                "mo" => dur.months = num,
                "w" => dur.weeks = num,
                "d" => dur.days = num,
                "h" => dur.hours = num,
                "m" => dur.minutes = num,
                "s" => dur.seconds = num,
                _ => {}
            }
        }

        dur
    }

    pub fn new_no_str(str: String) -> Self {
        let mut dur = Self::default();

        for cap in util::regex::DURATION.captures_iter(&str) {
            let num = cap[1].parse::<i64>().unwrap();
            let unit = &cap[2];

            match unit {
                "y" => dur.years = num,
                "mo" => dur.months = num,
                "w" => dur.weeks = num,
                "d" => dur.days = num,
                "h" => dur.hours = num,
                "m" => dur.minutes = num,
                "s" => dur.seconds = num,
                _ => {}
            }
        }

        dur
    }

    pub fn to_seconds(&self) -> i64 {
        let mut seconds = 0;

        seconds += self.years * 31536000;
        seconds += self.months * 2592000;
        seconds += self.weeks * 604800;
        seconds += self.days * 86400;
        seconds += self.hours * 3600;
        seconds += self.minutes * 60;
        seconds += self.seconds;

        seconds
    }

    pub fn to_unix_expiry(&self) -> Option<i64> {
        if self.is_permenant() {
            return None;
        }
        let now = chrono::Utc::now().timestamp();
        Some(now + self.to_seconds())
    }

    pub fn to_discord_timestamp(&self) -> String {
        let exp = self.to_unix_expiry();
        match exp {
            Some(exp) => format!("<t:{}:f>", exp),
            None => "`Never`".to_string(),
        }
    }

    pub fn to_discord_relative_timestamp(&self) -> String {
        let exp = self.to_unix_expiry();
        match exp {
            Some(exp) => format!("<t:{}:R>", exp),
            None => "`Never`".to_string(),
        }
    }

    pub fn is_permenant(&self) -> bool {
        self.years == 0
            && self.months == 0
            && self.weeks == 0
            && self.days == 0
            && self.hours == 0
            && self.minutes == 0
            && self.seconds == 0
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str: String = Deserialize::deserialize(deserializer)?;
        Ok(Duration::new(str))
    }
}

impl Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.full_string)
    }
}

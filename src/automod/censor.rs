use lazy_static::lazy_static;
use regex::*;

use crate::automod::*;
use crate::util::*;

lazy_static! {
    static ref DOMAINS_RE: Regex = Regex::new(r"[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}").unwrap();
    static ref IP_RE: Regex = Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}").unwrap();
}

pub fn filter_strings(censor_user : &automod::Censor, content: &String) -> (String, bool) {
    if !censor_user.filter_strings.unwrap_or(false) {
        return ("".to_string(), true);
    }

    match censor_user.blocked_strings.as_ref() {
        Some(blocked_strings) => {
            let content_split = content.split_whitespace();
            for word in content_split {
                if blocked_strings.contains(&word.to_string()) {
                    return (word.to_string(), false);
                }
            }
        },
        None => {}
    };

    match censor_user.blocked_substrings.as_ref() {
        Some(blocked_substrings) => {
            let nsp_content = clean::remove_spaces(&content);
            for string in blocked_substrings {
                if nsp_content.contains(string) {
                    return (string.to_string(), false);
                }
            }
        },
        None => {}
    };
        
    return ("".to_string(), true);
}

pub fn filter_invites(censor_user : &automod::Censor, content: &String) -> (String, bool) {
    match censor_user.invites_blacklist {
        Some(ref invites_blacklist) => {
            let (trigger, ok) = filter_blacklisted_invites(invites_blacklist, content);
            if !ok {
                return (trigger, false);
            }
        },
        None => ()
    }

    if !censor_user.filter_invites.unwrap_or(false) {
        return ("".to_string(), true);
    }
    let invites_whitelist = match censor_user.invites_whitelist.as_ref() {
        Some(invites_whitelist) => invites_whitelist,
        None => return ("".to_string(), true)
    };
    let content_split = content.split_whitespace();
    for word in content_split {
        if word.contains("discord.gg") {
            let invite_code = word.split("discord.gg/").collect::<Vec<&str>>()[1];
            if !invites_whitelist.contains(&invite_code.to_string()) || !invites_whitelist.contains(&format!("discord.gg/{}", invite_code)) {
                return (invite_code.to_string(), false);
            }
        }
    }
    return ("".to_string(), true);
}

pub fn filter_blacklisted_invites(invites_blacklist : &Vec<String>, content: &String) -> (String, bool) {
    let content_split = content.split_whitespace();
    for word in content_split {
        if word.contains("discord.gg") {
            let invite_code = word.split("discord.gg/").collect::<Vec<&str>>()[1];
            if invites_blacklist.contains(&invite_code.to_string()) || invites_blacklist.contains(&format!("discord.gg/{}", invite_code)) {
                return (invite_code.to_string(), false);
            }
        }
    }
    return ("".to_string(), true);
}

pub fn filter_domains(censor_user : &automod::Censor, content: &String) -> (String, bool) {
    match censor_user.domain_blacklist {
        Some(ref domain_blacklist) => {
            let (trigger, ok) = filter_blacklisted_domains(domain_blacklist, content);
            if !ok {
                return (trigger, false);
            }
        },
        None => ()
    }

    if !censor_user.filter_domains.unwrap_or(false) {
        return ("".to_string(), true);
    }

    let domains = DOMAINS_RE.find(&content);
    match domains {
        Some(domain) => {
            let domain_whitelist = match censor_user.domain_whitelist.as_ref() {
                Some(domain_whitelist) => domain_whitelist,
                None => return ("".to_string(), true)
            };
            if !domain_whitelist.contains(&domain.as_str().to_string()) {
                return (domain.as_str().to_string(), false);
            }
        }
        None => {}
    }
    return ("".to_string(), true);
}

pub fn filter_blacklisted_domains(blacklist : &Vec<String>, content: &String) -> (String, bool) {
    if blacklist.is_empty() {
        return ("".to_string(), true);
    }
    
    for domain in DOMAINS_RE.find_iter(&content) {
        if DOMAINS_RE.is_match(&domain.as_str()) {
            let mut domain = domain.as_str();
            if domain.starts_with("www.") {
                domain = domain.split("www.").collect::<Vec<&str>>()[1];
            }
            if blacklist.contains(&domain.to_string()) {
                return (domain.to_string(), false);
            }
        }
    }
    return ("".to_string(), true);
}

pub fn filter_ips(censor_user : &automod::Censor, content: &String) -> (String, bool) {
    if !censor_user.filter_ips.unwrap_or(false) {
        return ("".to_string(), true);
    }

    let ips = IP_RE.find_iter(&content);
    for ip in ips {
        let ip_vec = ip.as_str().split(".").collect::<Vec<&str>>();
        if ip_vec.len() != 4 { // ? wtf
            return ("".to_string(), true);
        }
        let ip = ip_vec.iter().map(|octet| octet.parse::<u8>().unwrap()).collect::<Vec<u8>>();
        if !ip::ip_full_check(&ip) {
            return (ip_vec.join(".").as_str().to_string(), false);
        }
    }

    return ("".to_string(), true);
}

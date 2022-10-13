pub fn ip_is_private(ip: &Vec<u8>) -> bool  {
    return ip[0] == 10 ||
    (ip[0] == 172 && ip[1]&0xf0 == 16) ||
    (ip[0] == 192 && ip[1] == 168);
}

pub fn ip_is_loopback(ip: &Vec<u8>) -> bool  {
    return ip[0] == 127;
}

pub fn ip_is_multicast(ip: &Vec<u8>) -> bool  {
    return ip[0] >= 224 && ip[0] <= 239;
}

pub fn ip_is_link_local_multicast(ip: &Vec<u8>) -> bool  {
    return ip[0] == 224 && ip[1] == 0 && ip[2] == 0;
}

pub fn ip_is_link_local_unicast(ip: &Vec<u8>) -> bool  {
    return ip[0] == 169 && ip[1] == 254;
}

pub fn ip_full_check(ip: &Vec<u8>) -> bool  {
    return ip_is_private(&ip) || ip_is_loopback(&ip) || ip_is_multicast(&ip) || ip_is_link_local_multicast(&ip) || ip_is_link_local_unicast(&ip);
}
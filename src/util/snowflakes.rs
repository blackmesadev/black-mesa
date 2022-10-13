pub fn snowflake_to_unix<T>(snowflake: T) -> u64 
where T: Into<u64>{
    (snowflake.into() >> 22) + 1420070400000
}
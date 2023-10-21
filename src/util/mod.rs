pub mod duration;
pub mod ip;
pub mod mentions;
pub mod permissions;
pub mod regex;
pub mod snowflakes;
pub mod unicode;

pub fn format_username(name: &str, discriminator: u16) -> String {
    if discriminator == 0 {
        return name.to_string();
    }
    format!("{}#{}", name, discriminator)
}

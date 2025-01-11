pub mod commands;
mod embed;
mod error;
mod guild;
mod audio;
mod model;
mod permissions;
mod rest;
mod ws;

pub use embed::*;
pub use error::{DiscordError, DiscordResult};
pub use model::*;
pub use permissions::*;
pub use rest::DiscordRestClient;
pub use ws::{DiscordWebsocket, Event};

pub const DISCORD_EPOCH: u64 = 1420070400000;

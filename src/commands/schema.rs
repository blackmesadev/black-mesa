pub const USER_TARGET: &str = "<target:user|id[]>";
pub const UUID_TARGET: &str = "<target:uuid[]>";
pub const PERMISSION_GROUP: &str = "<group:text>";

pub const PREFIX: &str = "<prefix:text>";
pub const ADD_ALIAS: &str = "<alias:command> <command:command>";
pub const REMOVE_ALIAS: &str = "<alias:command>";

pub const SET_CONFIG: &str = "<key:text> <value:text>";

pub const AUDIO_PLAYER_ID: &str = "<player_id:channel_id>";
pub const AUDIO_CONNECTION: &str = "<channel:id> <session_id:text> <token:text> <endpoint:text> [udp_server_addr:text] [udp_ssrc:number] [udp_key_hex:text]";
pub const AUDIO_PLAYER_CONNECTION: &str = "<channel:id> <session_id:text> <token:text> <endpoint:text> [udp_server_addr:text] [udp_ssrc:number] [udp_key_hex:text]";
pub const AUDIO_ENQUEUE: &str = "<url:text> [player_id:channel_id]";
pub const AUDIO_PLAYLIST: &str = "<name:text> [player_id:channel_id]";
pub const AUDIO_SEEK: &str = "<position_ms:number> [player_id:channel_id]";
pub const AUDIO_VOLUME: &str = "<volume:0.0-2.0> [player_id:channel_id]";

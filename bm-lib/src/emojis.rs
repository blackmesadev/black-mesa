pub enum Emoji {
    Kick,
    Ban,
    Mute,
    Warn,

    Unban,
    Unmute,
    Pardon,

    MessageDelete,
    MessageEdit,

    MemberJoin,
    MemberLeave,

    VoiceJoin,
    VoiceLeave,
    VoiceMove,

    CensoredMessage,
    MessageViolation,

    Check,
    Cross,
    Loading,
}

impl Emoji {
    #[inline]
    pub fn to_emoji(&self) -> &'static str {
        match self {
            Self::Kick => "<:mesaKick:869665034312253460>",
            Self::Ban => "<:mesaBan:869663336625733634>",
            Self::Mute => "<:mesaMemberMute:869663336814497832>",
            Self::Warn => "<:mesaWarn:869663336843845752>",

            Self::Unban => "<:mesaUnban:869663336697069619>",
            Self::Unmute => "<:mesaUnmute:869663336583802982>",
            Self::Pardon => "<:mesaPardon:869664457788358716>",

            Self::MessageDelete => "<:mesaMessageDelete:869663511977025586>",
            Self::MessageEdit => "<:mesaMessageEdit:869663511834411059>",

            Self::MemberJoin => "<:mesaMemberJoin:832350526896734248>",
            Self::MemberLeave => "<:mesaMemberLeave:832350526778900574>",

            Self::VoiceJoin => "<:mesaVoiceJoin:832350526636294165>",
            Self::VoiceLeave => "<:mesaVoiceLeave:832350526803935243>",
            Self::VoiceMove => "<:mesaVoiceMove:832350526883495940>",

            Self::CensoredMessage => "<:mesaCensoredMessage:869663511754731541>",
            Self::MessageViolation => "<:mesaMessageViolation:869663336625733635>",

            Self::Check => "<:mesaCheck:832350526729224243>",
            Self::Cross => "<:mesaCross:832350526414127195>",
            Self::Loading => "<:mesaLoading:832350526905516082>",
        }
    }
}

impl core::fmt::Display for Emoji {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_emoji())
    }
}

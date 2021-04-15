package config

type WebAccess struct {
	Admin  *[]string
	Editor *[]string
	Viewer *[]string
}

type Permissions struct {
	Guild      int64
	Censor     int64
	Moderation int64
}

type Commands struct {
	Prefix      string
	Permissions *Permissions
}

type persistance struct {
	Roles             bool
	WhitelistedRRoles *[]string // slice of ids
	Nickname          bool
	Voice             bool
}

type reactRoleEmote struct {
	Role string
}

type reactRoleChannel struct {
	Emotes map[string]*reactRoleEmote // emojiID : reactRoleEmote
}

type reactRoles struct {
	Channel map[string]*reactRoleChannel // channelid : reactRoleChannel
}

type Guild struct {
	ConfirmActions      bool
	RoleAliases         *map[string]string // name: roleid
	SelfAssignableRoles *map[string]string // name: roleid
	LockedRoles         *[]string          // slice of ids
	Persistance         *persistance
	AutoRole            *[]string // slice of ids
	ReactRoles          *reactRoles
}

type censor struct {
	FilterZalgo           bool
	FilterInvites         bool
	InvitesGuildWhitelist *[]string // slice of guildids
	InvitesGuildBlacklist *[]string // slice of guildids
	InvitesWhitelist      *[]string // slice of invitelinks
	InvitesBlacklist      *[]string // slice of invitelinks
	DomainWhitelist       *[]string // slice of domains
	DomainBlacklist       *[]string // slice of domains
	BlockedSubstrings     *[]string // slice of substrings
	BlockedStrings        *[]string // slice of strings
	Regex                 string
}

type spam struct {
	Punishment         string
	PunishmentDuration int64
	Count              int64 // amount per interval
	Interval           int64 // seconds
	MaxMessages        int64
	MaxMentions        int64
	MaxLinks           int64
	MaxAttachments     int64
	MaxEmojis          int64
	MaxNewlines        int64
	MaxDuplicates      int64
	Clean              bool
	CleanCount         int64
	CleanDuration      int64
}

type Automod struct {
	CensorLevels   *map[int64]*censor
	CensorChannels *map[string]*censor

	SpamLevels   *map[int64]*spam
	SpamChannels *map[string]*spam
}

type loggingChannel struct {
	IncludeActions *[]string // list of actions
	ExcludeActions *[]string // list of actions
	Timestamps     bool
	Timezone       string
}

type Logging struct {
	IgnoredUsers       *[]string // slice of user ids
	IgnoredChannels    *[]string // slice of channel ids
	NewMemberThreshold int64     // seconds
	Channels           *loggingChannel
}

type Moderation struct {
	ConfirmActionsMessage       bool
	ConfirmActionsMessageExpiry int64
	ConfirmActionsReaction      bool
	MuteRole                    string
	ReasonEditLevel             int64
	NotifyActions               *[]string
	ShowModeratorOnNotify       bool
	SilenceLevel                int64
}

type Modules struct {
	Guild      *Guild
	Automod    *Automod
	Logging    *Logging
	Moderation *Moderation
}

type Config struct {
	Nickname string

	WebAccess *WebAccess        `json:"webAccess" bson:"webAccess"`
	Commands  *Commands         `json:"commands" bson:"commands"`
	Levels    *map[string]int64 `json:"levels" bson:"levels"`

	Modules *Modules
}

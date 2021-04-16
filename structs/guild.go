package structs

type WebAccess struct {
	Admin  *[]string
	Editor *[]string
	Viewer *[]string
}

type Permissions struct {
	Guild          int64
	Administration int64
	Moderation     int64
	Roles          int64
	Logging        int64
}

type Commands struct {
	Prefix      string
	Permissions *Permissions
}

type Persistance struct {
	Roles            bool
	WhitelistedRoles *[]string // slice of ids
	Nickname         bool
	Voice            bool
}

type ReactRoleEmote struct {
	Role string
}

type ReactRoleChannel struct {
	Emotes map[string]*ReactRoleEmote // emojiID : reactRoleEmote
}

type ReactRoles struct {
	Channel map[string]*ReactRoleChannel // channelid : reactRoleChannel
}

type Guild struct {
	ConfirmActions      bool
	RoleAliases         map[string]string // name: roleid
	SelfAssignableRoles map[string]string // name: roleid
	LockedRoles         *[]string         // slice of ids
	Persistance         *Persistance
	AutoRole            *[]string // slice of ids
	ReactRoles          *ReactRoles
}

type Censor struct {
	FilterZalgo       bool
	FilterInvites     bool
	InvitesWhitelist  *[]string // slice of invitelinks/ids
	InvitesBlacklist  *[]string // slice of invitelinks/ids
	DomainWhitelist   *[]string // slice of domains
	DomainBlacklist   *[]string // slice of domains
	BlockedSubstrings *[]string // slice of substrings
	BlockedStrings    *[]string // slice of strings
	Regex             string
}

type Spam struct {
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
	CensorLevels   map[int64]*Censor
	CensorChannels map[string]*Censor

	SpamLevels   map[int64]*Spam
	SpamChannels map[string]*Spam

	PublicHumilation bool
}

type Logging struct {
	ChannelID          string
	IncludeActions     *[]string // list of actions
	ExcludeActions     *[]string // list of actions
	Timestamps         bool
	Timezone           string
	IgnoredUsers       *[]string // slice of user ids
	IgnoredChannels    *[]string // slice of channel ids
	NewMemberThreshold int64     // seconds
}

type Moderation struct {
	ConfirmActionsMessage       bool
	ConfirmActionsMessageExpiry int64
	ConfirmActionsReaction      bool
	MuteRole                    string
	ReasonEditLevel             int64
	NotifyActions               bool
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
	GuildID  string
	Nickname string

	WebAccess *WebAccess       `json:"webAccess" bson:"webAccess"`
	Commands  *Commands        `json:"commands" bson:"commands"`
	Levels    map[string]int64 `json:"levels" bson:"levels"`

	Modules *Modules
}

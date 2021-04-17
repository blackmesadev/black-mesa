package structs

type WebAccess struct {
	Admin  *[]string
	Editor *[]string
	Viewer *[]string
}

type Commands struct {
}

type Persistance struct {
	Roles            bool
	WhitelistedRoles *[]string `json:"whitelistedRoles" bson:"whitelistedRoles"` // slice of ids
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
	Channel map[string]*ReactRoleChannel // channelID : reactRoleChannel
}

type Guild struct {
	ConfirmActions      bool              `json:"confirmActions" bson:"confirmActions"`
	RoleAliases         map[string]string `json:"roleAliases" bson:"roleAliases"`                 // name: roleid
	SelfAssignableRoles map[string]string `json:"selfAssignableRoles" bson:"selfAssignableRoles"` // name: roleid
	LockedRoles         *[]string         `json:"lockedRoles" bson:"lockedRoles"`                 // slice of ids
	Persistance         *Persistance      `json:"persistance" bson:"persistance"`
	AutoRole            *[]string         `json:"autoRole" bson:"autoRole"` // slice of ids
	ReactRoles          *ReactRoles       `json:"reactRoles" bson:"reactRoles"`
	UnsafePermissions   bool              `json:"unsafePermissions" bson:"unsafePermissions"`
}

type Censor struct {
	FilterZalgo       bool      `json:"filterZalgo" bson:"filterZalgo"`
	FilterInvites     bool      `json:"filterInvites" bson:"filterInvites"`
	InvitesWhitelist  *[]string `json:"invitesWhitelist" bson:"invitesWhitelist"`   // slice of invitelinks/ids
	InvitesBlacklist  *[]string `json:"invitesBlacklist" bson:"invitesBlacklist"`   // slice of invitelinks/ids
	DomainWhitelist   *[]string `json:"domainWhitelist" bson:"domainWhitelist"`     // slice of domains
	DomainBlacklist   *[]string `json:"domainBlacklist" bson:"domainBlacklist"`     // slice of domains
	BlockedSubstrings *[]string `json:"blockedSubstrings" bson:"blockedSubstrings"` // slice of substrings
	BlockedStrings    *[]string `json:"blockedStrings" bson:"blockedStrings"`       // slice of strings
	Regex             string    `json:"regex" bson:"regex"`
}

type Spam struct {
	Punishment         string
	PunishmentDuration int64 `json:"punishmentDuration" bson:"punishmentDuration"` // seconds
	Count              int64 `json:"count" bson:"count"`                           // amount per interval
	Interval           int64 `json:"interval" bson:"interval"`                     // seconds
	MaxMessages        int64 `json:"maxMessages" bson:"maxMessages"`
	MaxMentions        int64 `json:"maxMentions" bson:"maxMentions"`
	MaxLinks           int64 `json:"maxLinks" bson:"maxLinks"`
	MaxAttachments     int64 `json:"maxAttachments" bson:"maxAttachments"`
	MaxEmojis          int64 `json:"maxEmojis" bson:"maxEmojis"`
	MaxNewlines        int64 `json:"maxNewlines" bson:"maxNewlines"`
	MaxDuplicates      int64 `json:"maxDuplicates" bson:"maxDuplicates"`
	Clean              bool  `json:"clean" bson:"clean"`
	CleanCount         int64 `json:"cleanCount" bson:"cleanCount"`
	CleanDuration      int64 `json:"cleanDuration" bson:"cleanDuration"`
}

type Automod struct {
	CensorLevels     map[int64]*Censor  `json:"censorLevels" bson:"censorLevels"`
	CensorChannels   map[string]*Censor `json:"censorChannels" bson:"censorChannels"`
	SpamLevels       map[int64]*Spam    `json:"spamLevels" bson:"spamLevels"`
	SpamChannels     map[string]*Spam   `json:"spamChannels" bson:"spamChannels"`
	PublicHumilation bool               `json:"publicHumilation" bson:"publicHumilation"`
}

type Logging struct {
	ChannelID          string    `json:"channelID" bson:"channelID"`
	IncludeActions     *[]string `json:"includeActions" bson:"includeActions"` // list of actions
	ExcludeActions     *[]string `json:"excludeActions" bson:"excludeActions"` // list of actions
	Timestamps         bool      `json:"timestamps" bson:"timestamps"`
	Timezone           string    `json:"timezone" bson:"timezone"`
	IgnoredUsers       *[]string `json:"ignoredUsers" bson:"ignoredUsers"`             // slice of user ids
	IgnoredChannels    *[]string `json:"ignoredChannels" bson:"ignoredChannels"`       // slice of channel ids
	NewMemberThreshold int64     `json:"newMemberThreshold" bson:"newMemberThreshold"` // seconds
}

type Moderation struct {
	ConfirmActionsMessage       bool   `json:"confirmActionsMessage" bson:"confirmActionsMessage"`
	ConfirmActionsMessageExpiry int64  `json:"confirmActionsMessageExpiry" bson:"confirmActionsMessageExpiry"`
	ConfirmActionsReaction      bool   `json:"confirmActionsReaction" bson:"confirmActionsReaction"`
	MuteRole                    string `json:"muteRole" bson:"muteRole"`
	ReasonEditLevel             int64  `json:"reasonEditLevel" bson:"reasonEditLevel"`
	NotifyActions               bool   `json:"notifyActions" bson:"notifyActions"`
	ShowModeratorOnNotify       bool   `json:"showModeratorOnNotify" bson:"showModeratorOnNotify"`
	SilenceLevel                int64  `json:"silenceLevel" bson:"silenceLevel"`
}

type Modules struct {
	Guild      *Guild
	Automod    *Automod
	Logging    *Logging
	Moderation *Moderation
}

type Config struct {
	guildID     string
	Nickname    string           `json:"nickname" bson:"nickname"`
	WebAccess   *WebAccess       `json:"webAccess" bson:"webAccess"`
	Prefix      string           `json:"prefix" bson:"prefix"`
	Permissions map[string]int64 `json:"permissions" bson:"permissions"`
	Levels      map[string]int64 `json:"levels" bson:"levels"`
	Modules     *Modules         `json:"modules" bson:"modules"`
}

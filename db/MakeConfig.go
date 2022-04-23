package db

import (
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func MakeConfig(g *discordgo.Guild, invokedByUserID string) *structs.Config {
	admins := make([]string, 0, 2)
	admins = append(admins, g.OwnerID)

	wa := &structs.WebAccess{
		Admin: admins,
	}
	lvls := make(map[string]int64)
	lvls[g.OwnerID] = 100

	if g.OwnerID != invokedByUserID {
		admins = append(admins, invokedByUserID)
		lvls[invokedByUserID] = 100
	}

	perms := make(map[string]int64)
	perms[consts.CATEGORY_MODERATION] = 50
	perms[consts.CATEGORY_ADMIN] = 100
	perms[consts.CATEGORY_GUILD] = 100
	perms[consts.CATEGORY_ROLES] = 100
	perms[consts.CATEGORY_MUSIC] = 10

	emptyMap := make(map[string]string)
	emptySlice := make([]string, 0)

	persist := &structs.Persistance{
		Roles:            false,
		WhitelistedRoles: emptySlice,
		Nickname:         false,
		Voice:            false,
	}

	reactRoles := &structs.ReactRoles{
		Channel: make(map[string]*structs.ReactRoleChannel),
	}

	// fill structs.guild
	guild := &structs.Guild{
		ConfirmActions:      false,
		RoleAliases:         emptyMap,
		SelfAssignableRoles: emptyMap,
		LockedRoles:         emptySlice,
		Persistance:         persist,
		AutoRole:            emptySlice,
		ReactRoles:          reactRoles,
		UnsafePermissions:   false,
		StaffLevel:          50,
	}
	defaultSpam := &structs.Spam{
		Punishment:          "N0NE",
		PunishmentDuration:  0,
		Count:               0,
		Interval:            0,
		MaxMessages:         0,
		MaxMentions:         0,
		MaxLinks:            0,
		MaxAttachments:      0,
		MaxEmojis:           0,
		MaxNewlines:         0,
		MaxDuplicates:       0,
		MaxCharacters:       0,
		MaxUppercasePercent: 0,
		MinUppercaseLimit:   0,
		Clean:               false,
		CleanCount:          0,
		CleanDuration:       0,
	}

	defaultCensor := &structs.Censor{
		FilterZalgo:            true,
		FilterInvites:          true,
		FilterDomains:          false,
		FilterStrings:          true,
		FilterIPs:              false,
		FilterRegex:            false,
		FilterEnglish:          false,
		FilterObnoxiousUnicode: false,
		FilterUntrustworthy:    true,
		InvitesWhitelist:       emptySlice,
		InvitesBlacklist:       emptySlice,
		DomainWhitelist:        emptySlice,
		DomainBlacklist:        emptySlice,
		BlockedSubstrings:      emptySlice,
		BlockedStrings:         emptySlice,
		Regex:                  "",
	}

	censorlvls := make(map[int64]*structs.Censor)
	censorlvls[0] = defaultCensor

	spamlvls := make(map[int64]*structs.Spam)
	spamlvls[0] = defaultSpam

	automod := &structs.Automod{
		Enabled: false,
		GuildOptions: &structs.GuildOptions{
			MinimumAccountAge: "1w",
		},
		CensorLevels:   censorlvls,
		CensorChannels: make(map[string]*structs.Censor),
		SpamLevels:     spamlvls,
		SpamChannels:   make(map[string]*structs.Spam),

		PublicHumilation: false,
		StaffBypass:      true,
	}

	logs := &structs.Logging{
		Enabled:            false,
		ChannelID:          "",
		IncludeActions:     emptySlice,
		ExcludeActions:     emptySlice,
		Timestamps:         true,
		Timezone:           "GMT",
		IgnoredUsers:       emptySlice,
		IgnoredChannels:    emptySlice,
		NewMemberThreshold: 0,
	}

	strikeEscalation := make(map[int64]structs.StrikeEscalation)

	moderation := &structs.Moderation{
		CensorSearches:              true,
		CensorStaffSearches:         true,
		ConfirmActionsMessage:       true,
		ConfirmActionsMessageExpiry: 0,
		ConfirmActionsReaction:      false,
		DisplayNoPermission:         true,
		MuteRole:                    "",
		ReasonEditLevel:             50,
		NotifyActions:               true,
		ShowModeratorOnNotify:       true,
		SilenceLevel:                100,
		StrikeEscalation:            strikeEscalation,
		StrikeCushioning:            3,
	}

	memberRemove := make(map[int64]structs.AntiNukeThreshold, 0)
	memberRemove[0] = structs.AntiNukeThreshold{
		Max:      5,
		Interval: 10,
		Type:     "ban",
	}
	an := &structs.AntiNuke{
		Enabled:      false,
		MemberRemove: memberRemove,
	}

	vote := &structs.Voting{
		VoteMute: &structs.VoteMute{
			Enabled:         false,
			MaxDuration:     600,
			UpvotesRequired: 10,
		},
	}

	mods := &structs.Modules{
		Guild:      guild,
		Automod:    automod,
		Logging:    logs,
		Moderation: moderation,
		AntiNuke:   an,
		Voting:     vote,
	}

	config := &structs.Config{
		Nickname: "Black Mesa",

		WebAccess:   wa,
		Prefix:      "!",
		Permissions: perms,
		Levels:      lvls,
		Modules:     mods,
	}

	return config
}

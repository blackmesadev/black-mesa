package config

import (
	"github.com/bwmarrin/discordgo"
	"github.com/trollrocks/black-mesa/structs"
)

func MakeConfig(g *discordgo.Guild, invokedByUserID string) *structs.Config {
	admins := make([]string, 0, 2)
	admins = append(admins, g.OwnerID)

	wa := &structs.WebAccess{
		Admin: &admins,
	}
	lvls := make(map[string]int64)
	lvls[g.OwnerID] = 100

	if g.OwnerID != invokedByUserID {
		admins = append(admins, invokedByUserID)
		lvls[invokedByUserID] = 100
	}

	perms := &structs.Permissions{
		Guild:          0,
		Administration: 100,
		Moderation:     50,
		Roles:          100,
		Logging:        100,
	}

	cmds := &structs.Commands{
		Prefix:      "!",
		Permissions: perms,
	}
	emptyMap := make(map[string]string)
	emptySlice := make([]string, 0, 0)

	persist := &structs.Persistance{Roles: false,
		WhitelistedRoles: &emptySlice,
		Nickname:         false,
		Voice:            false,
	}

	reactRoles := &structs.ReactRoles{
		Channel: make(map[string]*structs.ReactRoleChannel),
	}

	guild := &structs.Guild{
		ConfirmActions:      false,
		RoleAliases:         emptyMap,
		SelfAssignableRoles: emptyMap,
		LockedRoles:         &emptySlice,
		Persistance:         persist,
		AutoRole:            &emptySlice,
		ReactRoles:          reactRoles,
	}
	defaultSpam := &structs.Spam{
		Punishment:         "N0NE",
		PunishmentDuration: 0,
		Count:              0,
		Interval:           0,
		MaxMessages:        0,
		MaxMentions:        0,
		MaxLinks:           0,
		MaxAttachments:     0,
		MaxEmojis:          0,
		MaxNewlines:        0,
		MaxDuplicates:      0,
		Clean:              false,
		CleanCount:         0,
		CleanDuration:      0,
	}

	defaultCensor := &structs.Censor{
		FilterZalgo:       true,
		FilterInvites:     true,
		InvitesWhitelist:  &emptySlice,
		InvitesBlacklist:  &emptySlice,
		DomainWhitelist:   &emptySlice,
		DomainBlacklist:   &emptySlice,
		BlockedSubstrings: &emptySlice,
		BlockedStrings:    &emptySlice,
		Regex:             "",
	}

	censorlvls := make(map[int64]*structs.Censor)
	censorlvls[0] = defaultCensor

	spamlvls := make(map[int64]*structs.Spam)
	spamlvls[0] = defaultSpam

	automod := &structs.Automod{
		CensorLevels:   censorlvls,
		CensorChannels: make(map[string]*structs.Censor),
		SpamLevels:     spamlvls,
		SpamChannels:   make(map[string]*structs.Spam),

		PublicHumilation: false,
	}

	logs := &structs.Logging{
		ChannelID:          "",
		IncludeActions:     &emptySlice,
		ExcludeActions:     &emptySlice,
		Timestamps:         true,
		Timezone:           "GMT",
		IgnoredUsers:       &emptySlice,
		IgnoredChannels:    &emptySlice,
		NewMemberThreshold: 0,
	}

	moderation := &structs.Moderation{
		ConfirmActionsMessage:       true,
		ConfirmActionsMessageExpiry: 0,
		ConfirmActionsReaction:      false,
		MuteRole:                    "",
		ReasonEditLevel:             50,
		NotifyActions:               true,
		ShowModeratorOnNotify:       true,
		SilenceLevel:                100,
	}

	mods := &structs.Modules{
		Guild:      guild,
		Automod:    automod,
		Logging:    logs,
		Moderation: moderation,
	}

	config := &structs.Config{
		Nickname: "Black Mesa",

		WebAccess: wa,
		Commands:  cmds,
		Levels:    lvls,

		Modules: mods,
	}

	return config
}

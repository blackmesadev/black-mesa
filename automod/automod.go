package automod

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

func Process(s *discordgo.Session, m *discordgo.Message) {
	ok, reason := Check(s, m)
	if !ok {
		msg := fmt.Sprintf(m.Content, ok, reason)
		s.ChannelMessageSend(m.ChannelID, msg)
	}
}

// Return true if all is okay, return false if not.
// This function should be "silent" if a message is okay.
func Check(s *discordgo.Session, m *discordgo.Message) (bool, string) {

	conf, err := config.GetConfig(m.GuildID)

	if conf == nil || err != nil {
		return true, ""
	}

	automod := conf.Modules.Automod

	content := m.Content

	if len(automod.SpamLevels) == 0 && len(automod.SpamChannels) == 0 &&
		len(automod.CensorLevels) == 0 && len(automod.SpamChannels) == 0 {
		return true, ""
	}

	censorChannel := automod.CensorChannels[m.ChannelID]

	// levels take priority
	userLevel := config.GetLevel(s, m.GuildID, m.Member.User.ID)
	levelCensor := automod.CensorLevels[userLevel]

	// Censor

	if userLevel > 0 {
		if levelCensor.FilterZalgo {
			ok := ZalgoCheck(content)
			if !ok {
				RemoveMessage(s, m)
				return false, "FilterZalgo"
			}
		}

	}

	// Censor

	// Zalgo
	if censorChannel.FilterZalgo {
		ok := ZalgoCheck(content)
		if !ok {
			RemoveMessage(s, m)
			return false, "FilterZalgo"
		}

	}

	// Invites
	if censorChannel.FilterInvites {
		ok := InvitesWhitelistCheck(content, censorChannel.InvitesWhitelist)
		if !ok {
			RemoveMessage(s, m)
			return false, "FilterZalgo"
		}

	} else if len(*censorChannel.InvitesBlacklist) != 0 {
		ok := InvitesBlacklistCheck(content, censorChannel.InvitesBlacklist)
		if !ok {
			RemoveMessage(s, m)
			return false, "FilterZalgo"
		}
	}

	// Domains

	return true, ""
}

package automod

import (
	"fmt"
	"math"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

// Gets the closest level that the ideal level can match in the level -> interface map
func getClosestLevel(i []int64, targetLevel int64) int64 {
	var closest int64 = 0
	for _, level := range i {
		if level == targetLevel {
			return targetLevel
		}

		if math.Abs(float64(targetLevel - level)) < math.Abs(float64(targetLevel - closest)) {
			closest = level
		}
	}

	return closest
}

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
		fmt.Println(conf, err)
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
	userLevel := config.GetLevel(s, m.GuildID, m.Author.ID)

	i := 0
	automodCensorLevels := make([]int64, len(automod.CensorLevels))
	for k := range automod.CensorLevels {
    	automodCensorLevels[i] = k
    	i++
	}

	levelCensor := automod.CensorLevels[getClosestLevel(automodCensorLevels, userLevel)]

	// Level censors
	if levelCensor != nil {
		// Zalgo
		if levelCensor.FilterZalgo {
			ok := ZalgoCheck(content)
			if !ok {
				RemoveMessage(s, m)
				return false, "FilterZalgo"
			}
		}

		// Invites
		if levelCensor.FilterInvites {
			ok := InvitesWhitelistCheck(content, levelCensor.InvitesWhitelist)
			if !ok {
				RemoveMessage(s, m)
				return false, "InvitesWhitelist"
			}
		} else if len(*levelCensor.InvitesBlacklist) != 0 {
			ok := InvitesBlacklistCheck(content, levelCensor.InvitesBlacklist)
			if !ok {
				RemoveMessage(s, m)
				return false, "InvitesBlacklist"
			}
		}
	}

	// Channel censors
	if censorChannel != nil {
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
				return false, "InvitesWhitelist"
			}

		} else if len(*censorChannel.InvitesBlacklist) != 0 {
			ok := InvitesBlacklistCheck(content, censorChannel.InvitesBlacklist)
			if !ok {
				RemoveMessage(s, m)
				return false, "InvitesBlacklist"
			}
		}
	}

	// Domains

	return true, ""
}

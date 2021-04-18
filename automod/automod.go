package automod

import (
	"fmt"
	"time"

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

		if level < targetLevel {
			closest = level
		} else {
			return closest // micro optimization; return early if the level is ever higher than the target
		}
	}

	return closest
}

func Process(s *discordgo.Session, m *discordgo.Message) {
	start := time.Now()
	ok, reason, filterProcessingStart := Check(s, m)
	if !ok {
		filtersDone := time.Since(filterProcessingStart)
		RemoveMessage(s, m)
		msg := fmt.Sprintf("Removed message for %v in %v (filters done in %v)", reason, time.Since(start), filtersDone)
		s.ChannelMessageSend(m.ChannelID, msg)
	}
}

// Return true if all is okay, return false if not.
// This function should be "silent" if a message is okay.
func Check(s *discordgo.Session, m *discordgo.Message) (bool, string, time.Time) {
	filterProcessingStart := time.Now()

	conf, err := config.GetConfig(m.GuildID)

	if conf == nil || err != nil {
		fmt.Println(conf, err)
		return true, "", filterProcessingStart
	}

	automod := conf.Modules.Automod

	content := m.Content

	if len(automod.SpamLevels) == 0 && len(automod.SpamChannels) == 0 &&
		len(automod.CensorLevels) == 0 && len(automod.SpamChannels) == 0 {
		return true, "", filterProcessingStart
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
				return false, "FilterZalgo", filterProcessingStart
			}
		}

		// Invites
		if levelCensor.FilterInvites {
			ok := InvitesWhitelistCheck(content, levelCensor.InvitesWhitelist)
			if !ok {
				return false, "Invite", filterProcessingStart
			}
		} else if len(*levelCensor.InvitesBlacklist) != 0 {
			ok := InvitesBlacklistCheck(content, levelCensor.InvitesBlacklist)
			if !ok {
				return false, "InvitesBlacklist", filterProcessingStart
			}
		}
	}

	// Channel censors
	if censorChannel != nil {
		// Zalgo
		if censorChannel.FilterZalgo {
			ok := ZalgoCheck(content)
			if !ok {
				return false, "FilterZalgo", filterProcessingStart
			}
		}

		// Invites
		if censorChannel.FilterInvites {
			ok := InvitesWhitelistCheck(content, censorChannel.InvitesWhitelist)
			if !ok {
				return false, "InvitesWhitelist", filterProcessingStart
			}

		} else if len(*censorChannel.InvitesBlacklist) != 0 {
			ok := InvitesBlacklistCheck(content, censorChannel.InvitesBlacklist)
			if !ok {
				return false, "InvitesBlacklist", filterProcessingStart
			}
		}
	}

	// Domains

	return true, "", filterProcessingStart
}

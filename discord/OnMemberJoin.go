package discord

import (
	"github.com/blackmesadev/black-mesa/automod"
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMemberJoin(s *discordgo.Session, m *discordgo.GuildMemberAdd) {
	conf, err := config.GetConfig(m.GuildID)
	if err != nil {
		return
	}

	// Check if this user is trying to bypass a mute or something similar
	automod.ProcessGuildMemberAdd(s, m, conf)

}

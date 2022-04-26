package discord

import (
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/modules/automod"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMemberJoin(s *discordgo.Session, m *discordgo.GuildMemberAdd) {
	conf, err := db.GetConfig(m.GuildID)
	if err != nil || conf == nil {
		return
	}

	// Check if this user is trying to bypass a mute or something similar
	automod.ProcessGuildMemberAdd(s, m, conf)

}

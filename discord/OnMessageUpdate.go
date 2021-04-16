package discord

import (
	"github.com/blackmesadev/black-mesa/automod"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMessageUpdate(s *discordgo.Session, m *discordgo.MessageUpdate) {

	automod.Process(s, m.Message)
}

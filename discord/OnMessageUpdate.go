package discord

import (
	"github.com/blackmesadev/black-mesa/automod"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMessageUpdate(s *discordgo.Session, m *discordgo.MessageUpdate) {
	if m.BeforeUpdate != nil && m.Author != nil {
		logging.LogMessageUpdate(s, m.Message, m.BeforeUpdate.Content)
	} // not cached otherwise
	automod.Process(s, m.Message)
}

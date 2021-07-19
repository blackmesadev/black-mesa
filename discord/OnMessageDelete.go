package discord

import (
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMessageDelete(s *discordgo.Session, m *discordgo.MessageDelete) {
	if m.BeforeDelete != nil {
		logging.LogMessageDelete(s, m.BeforeDelete)
	} // not cached otherwise
}

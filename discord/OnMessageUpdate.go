package discord

import (
	"github.com/blackmesadev/black-mesa/automod"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMessageUpdate(s *discordgo.Session, m *discordgo.MessageUpdate) {
	// stop nil pointer exceptions
	if m == nil || m.Author == nil {
		return
	}
	if m.Author.Bot {
		return
	} // just ignore all bot messages, good bots don't need to be moderated by us
	if m.BeforeUpdate != nil {
		logging.LogMessageUpdate(s, m.Message, m.BeforeUpdate.Content)
		automod.Process(s, m.Message)
	} // not cached otherwise
}

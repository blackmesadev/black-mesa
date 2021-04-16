package discord

import (
	"github.com/bwmarrin/discordgo"
	"github.com/trollrocks/black-mesa/automod"
)

func (bot *Bot) messageUpdateHandler(s *discordgo.Session, m *discordgo.MessageUpdate) {

	go automod.Process(s, m.Message)
}

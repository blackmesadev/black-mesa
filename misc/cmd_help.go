package misc

import (
	"github.com/blackmesadev/discordgo"
)

func HelpCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	s.ChannelMessageSend(m.ChannelID, "Help can be found at blackmesawebsite")
}

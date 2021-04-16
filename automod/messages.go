package automod

import (
	"github.com/blackmesadev/discordgo"
)

func RemoveMessage(s *discordgo.Session, m *discordgo.Message) bool {
	err := s.ChannelMessageDelete(m.ChannelID, m.ID)
	if err != nil {
		return false
	}
	return true
}

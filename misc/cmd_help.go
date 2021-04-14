package misc

import "github.com/bwmarrin/discordgo"

func Help(s *discordgo.Session, m *discordgo.MessageCreate, parameters []string) {
	s.ChannelMessageSend(m.ChannelID, "Help can be found at blackmesawebsite")
}

package structs

import "github.com/blackmesadev/discordgo"

type PurgeStruct struct {
	Messages  []*discordgo.Message
	GuildID   string
	ChannelID string
	IssuerID  string
	UUID      string
}

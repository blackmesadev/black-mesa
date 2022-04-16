package structs

import "github.com/blackmesadev/discordgo"

type PurgeStruct struct {
	Messages  []*discordgo.Message `json:"messages,omitempty" bson:"messages,omitempty"`
	GuildID   string               `json:"guildID,omitempty" bson:"guildID,omitempty"`
	ChannelID string               `json:"channelID,omitempty" bson:"channelID,omitempty"`
	IssuerID  string               `json:"issuerID,omitempty" bson:"issuerID,omitempty"`
	UUID      string               `json:"uuid,omitempty" bson:"uuid,omitempty"`
}

package mongodb

import "github.com/blackmesadev/black-mesa/structs"

type MongoGuild struct {
	GuildID string          `json:"guildID" bson:"guildID"`
	Config  *structs.Config `json:"config" bson:"config"`
}

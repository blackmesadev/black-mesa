package mongodb

import "github.com/blackmesadev/black-mesa/structs"

type MongoGuild struct {
	GuildID string          `json:"guildID" bson:"guildID"`
	Config  *structs.Config `json:"config" bson:"config"`
}

type MongoExpiringPunishment struct {
	GuildID        string `bson:"guildID"`
	UserID         string `bson:"userID"`
	RoleID         string `bson:"roleID,omitempty"`
	PunishmentType string `bson:"punishmentType"`
	Expires        int64  `bson:"expires"`
}

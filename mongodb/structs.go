package mongodb

import "github.com/blackmesadev/black-mesa/structs"

type MongoGuild struct {
	GuildID string          `json:"guildID" bson:"guildID"`
	Config  *structs.Config `json:"config" bson:"config"`
}

type MongoPunishment struct {
	GuildID        string `bson:"guildID"`
	UserID         string `bson:"userID"`
	Issuer         string `bson:"issuer"`
	PunishmentType string `bson:"punishmentType"`
	Expires        int64  `bson:"expires,omitempty"`

	// punishment specific fields
	// mutes
	RoleID         string `bson:"roleID,omitempty"`

	// strikes
	Weight         int    `bson:"weight,omitempty"`
	Reason         string `bson:"reason,omitempty"`
}
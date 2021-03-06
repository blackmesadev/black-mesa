package db

import (
	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type MongoGuild struct {
	GuildID string          `json:"guildID" bson:"guildID"`
	Config  *structs.Config `json:"config" bson:"config"`
}

type Action struct {
	// for timestamps
	ID primitive.ObjectID `bson:"_id,omitempty"`

	// action data
	GuildID string `bson:"guildID"`
	UserID  string `bson:"userID"`
	Issuer  string `bson:"issuer"`
	Type    string `bson:"type"`
	Expires int64  `bson:"expires,omitempty"`

	// punishment specific fields
	// mutes
	RoleID string `bson:"roleID,omitempty"`

	// strikes
	Weight int64  `bson:"weight,omitempty"`
	Reason string `bson:"reason,omitempty"`

	UUID string `bson:"uuid"`
}

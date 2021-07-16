package config

import (
	"github.com/blackmesadev/black-mesa/mongodb"
	"go.mongodb.org/mongo-driver/mongo"
)

func AddPunishment(punishment *mongodb.MongoPunishment) (*mongo.InsertOneResult, error) {
	return db.AddPunishment(punishment)
}

func GetPunishments(guildid string, userid string) ([]*mongodb.MongoPunishment, error) {
	return db.GetPunishments(guildid, userid)
}

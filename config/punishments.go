package config

import (
	"github.com/blackmesadev/black-mesa/mongodb"
	"go.mongodb.org/mongo-driver/mongo"
)

func AddAction(punishment *mongodb.Action) (*mongo.InsertOneResult, error) {
	return db.AddAction(punishment)
}

func RemoveAction(guildid string, uuid string) (*mongo.DeleteResult, error) {
	return db.RemoveAction(guildid, uuid)
}

func GetPunishments(guildid string, userid string) ([]*mongodb.Action, error) {
	return db.GetPunishments(guildid, userid)
}

func GetActions(guildid string, userid string) ([]*mongodb.Action, error) {
	return db.GetActions(guildid, userid)
}

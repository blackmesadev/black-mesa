package config

import (
	"github.com/blackmesadev/black-mesa/mongodb"
	"go.mongodb.org/mongo-driver/mongo"
)

func AddPunishment(punishment *mongodb.MongoExpiringPunishment) (*mongo.InsertOneResult, error) {
	return db.AddPunishment(punishment)
}

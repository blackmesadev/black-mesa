package mongodb

import (
	"context"
	"time"

	"github.com/trollrocks/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
)

func (db *DB) GetConfig(id string) (*structs.Config, error) {
	var config *structs.Config
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildid": id}

	result := col.FindOne(ctx, filters)
	result.Decode(&config)

	return config, nil
}

func (db *DB) AddConfig(config *structs.Config) (*mongo.InsertOneResult, error) {
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	result, err := col.InsertOne(ctx, config)
	if err != nil {
		return nil, err
	}
	return result, nil
}

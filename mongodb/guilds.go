package mongodb

import (
	"context"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func (db *DB) GetConfig(id string) (*structs.Config, error) {
	var config *MongoGuild
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildid": id}

	result := col.FindOne(ctx, filters)
	result.Decode(&config)

	return config.Config, nil
}

func (db *DB) GetConfigProjection(id string, projection string) (*bson.M, error) {
	var data *bson.M
	if projection == "" {
		projection = "guild"
	} else {
		projection = "guild." + projection
	}

	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildid": id}

	opts := options.FindOne().SetProjection(&bson.M{projection: "$" + projection})

	result := col.FindOne(ctx, filters, opts)
	result.Decode(&data)

	return data, nil
}

func (db *DB) AddConfig(config *MongoGuild) (*mongo.InsertOneResult, error) {
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	result, err := col.InsertOne(ctx, config)
	if err != nil {
		return nil, err
	}
	return result, nil
}

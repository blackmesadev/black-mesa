package mongodb

import (
	"context"
	"log"
	"reflect"
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

	filters := &bson.M{"guildID": id}

	result := col.FindOne(ctx, filters)
	err := result.Decode(&config)
	if config == nil || err != nil {
		return nil, err
	}

	return config.Config, nil
}

func (db *DB) SetConfigOne(id string, key string, value interface{}) (*mongo.UpdateResult, error) {
	originalValue, err := db.GetConfigProjection(id, key)

	if err != nil {
		return nil, err
	}

	originalValueType := reflect.TypeOf(originalValue)

	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": id}

	key = "config." + key

	update := &bson.M{"$set": bson.M{key: reflect.ValueOf(value).Convert(originalValueType)}}

	results, err := col.UpdateOne(ctx, filters, update)
	if err != nil {
		log.Println(err)
		return nil, err
	}

	return results, nil
}

func (db *DB) GetConfigProjection(id string, projection string) (bson.M, error) {
	var data bson.M
	if projection == "" {
		projection = "config"
	} else {
		projection = "config." + projection
	}

	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": id}

	opts := options.FindOne().SetProjection(&bson.M{projection: "$" + projection})

	result := col.FindOne(ctx, filters, opts)
	result.Decode(&data)

	return data, nil
}

func (db *DB) GetConfigMultipleProjection(id string, projection []string) (bson.M, error) {
	var data bson.M
	var updatedProjections bson.D

	for _, v := range projection {
		v = "config." + v
		updatedProjections = append(updatedProjections, bson.E{Key: v, Value: "$" + v})
	}

	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": id}

	opts := options.FindOne().SetProjection(updatedProjections)

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

func (db *DB) AddPunishment(punishment *MongoPunishment) (*mongo.InsertOneResult, error) {
	col := db.client.Database("black-mesa").Collection("punishments")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	insert, err := bson.Marshal(punishment)
	if err != nil {
		log.Println(err)
		return nil, err
	}

	results, err := col.InsertOne(ctx, insert)
	if err != nil {
		log.Println(err)
		return nil, err
	}

	return results, nil
}

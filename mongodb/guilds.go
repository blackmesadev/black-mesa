package mongodb

import (
	"context"
	"fmt"
	"log"
	"reflect"
	"strconv"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

func cast(v string, originalValue interface{}) (interface{}, error) {
	switch originalValue.(type) {
	case string:
		return v, nil
	case int64:
		return strconv.ParseInt(v, 10, 64)
	case bool:
		return strconv.ParseBool(v)
	case float64:
		return strconv.ParseFloat(v, 64)
	default:
		return nil, fmt.Errorf("cannot convert %q to %q", v, reflect.TypeOf(originalValue).String())
	}
}

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

func (db *DB) SetConfigOne(id string, key string, value string) (*mongo.UpdateResult, error) {
	originalValue, err := db.GetConfigProjection(id, key)

	if err != nil {
		return nil, err
	}

	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": id}

	key = "config." + key

	castedValue, err := cast(value, originalValue)
	if err != nil {
		return nil, err
	}

	update := &bson.M{"$set": bson.M{key: castedValue}}

	results, err := col.UpdateOne(ctx, filters, update)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
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

func (db *DB) AddAction(punishment *Action) (*mongo.InsertOneResult, error) {
	col := db.client.Database("black-mesa").Collection("actions")
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

func (db *DB) GetPunishments(guildid string, userid string) ([]*Action, error) {
	var punishments []*Action

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"userID":  userid,
		"type": bson.M{
			"$ne": "role",
		},
	}

	cursor, err := col.Find(ctx, query)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	for cursor.Next(ctx) {
		doc := Action{}
		cursor.Decode(&doc)
		punishments = append(punishments, &doc)
	}
	return punishments, err
}

func (db *DB) RemoveAction(guildid string, uuid string) (*mongo.DeleteResult, error) {
	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"uuid":    uuid,
	}
	deleteResult, err := col.DeleteOne(ctx, query)
	if err != nil {
		return nil, err
	}
	return deleteResult, nil
}

func (db *DB) GetActions(guildid string, userid string) ([]*Action, error) {
	var actions []*Action

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"userID":  userid,
	}

	cursor, err := col.Find(ctx, query)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	for cursor.Next(ctx) {
		doc := Action{}
		cursor.Decode(&doc)
		actions = append(actions, &doc)
	}
	return actions, err
}

func (db *DB) GetNonPunishments(guildid string, userid string) ([]*Action, error) {
	var actions []*Action

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"userID":  userid,
		"type":    "role", // its only temp roles that are not punishments as of now so this is just easier
	}

	cursor, err := col.Find(ctx, query)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	for cursor.Next(ctx) {
		doc := Action{}
		cursor.Decode(&doc)
		actions = append(actions, &doc)
	}
	return actions, err
}

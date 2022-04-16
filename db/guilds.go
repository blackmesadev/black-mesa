package db

import (
	"context"
	"log"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
)

func GetConfig(id string) (*structs.Config, error) {
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

func AddConfig(config *MongoGuild) (*mongo.InsertOneResult, error) {
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	return col.InsertOne(ctx, config)
}

func SetConfig(id string, config *structs.Config) (*mongo.UpdateResult, error) {
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": id}

	return col.UpdateOne(ctx, filters, config)
}

func AddAction(punishment *Action) (*mongo.InsertOneResult, error) {
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

func GetPunishments(guildid string, userid string) ([]*Action, error) {
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

func GetBan(guildid string, userid string) (*Action, error) {
	var ban *Action

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"userID":  userid,
		"type":    "ban",
	}

	cursor, err := col.Find(ctx, query)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	for cursor.Next(ctx) {
		cursor.Decode(&ban)
	}
	return ban, err
}

func GetMute(guildid string, userid string) (*Action, error) {
	var mute *Action

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"userID":  userid,
		"type":    "mute",
	}

	cursor, err := col.Find(ctx, query)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	for cursor.Next(ctx) {
		cursor.Decode(&mute)
	}
	return mute, err
}

func GetPunishmentByUUID(guildid string, uuid string) (*Action, error) {
	var action *Action

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"uuid":    uuid,
	}

	cursor, err := col.Find(ctx, query)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	for cursor.Next(ctx) {
		cursor.Decode(&action)
	}
	return action, err
}

func DeleteMute(guildid string, userid string) (*mongo.DeleteResult, error) {

	col := db.client.Database("black-mesa").Collection("actions")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	query := bson.M{
		"guildID": guildid,
		"userID":  userid,
		"type":    "mute",
	}

	result, err := col.DeleteMany(ctx, query)

	return result, err
}

func RemoveAction(guildid string, uuid string) (*mongo.DeleteResult, error) {
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

func GetActions(guildid string, userid string) ([]*Action, error) {
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

func GetNonPunishments(guildid string, userid string) ([]*Action, error) {
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

// Set funcs
func SetMutedRole(guildid string, roleid string) (*mongo.UpdateResult, error) {
	col := db.client.Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	return col.UpdateOne(ctx,
		bson.M{
			"guildID": guildid,
		},
		bson.M{
			"$set": bson.M{
				"dbmodules.moderation.muteRole": roleid,
			},
		},
	)
}

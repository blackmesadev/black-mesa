package db

import (
	"context"
	"log"

	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
)

type DB struct {
	client *mongo.Client
}

func InitDB() *DB {
	db := &DB{client: &mongo.Client{}}
	return db
}

func (db *DB) ConnectDB(mongoCfg structs.MongoConfig) {
	var err error

	creds := options.Credential{}
	creds.Username = mongoCfg.Username
	creds.Password = mongoCfg.Password
	clientOptions := options.Client()
	clientOptions.ApplyURI(mongoCfg.ConnectionString)

	if mongoCfg.Username != "" && mongoCfg.Password != "" {
		clientOptions.SetAuth(creds)
	}

	db.client, err = mongo.Connect(context.TODO(), clientOptions)

	if err != nil {
		log.Fatalln("Mongo Connection Failed. Unable to start, ", err)
	}

	err = db.client.Ping(context.TODO(), nil)

	if err != nil {
		log.Fatalln("Mongo Connection Failed. Unable to start, ", err)
	}

	log.Println("Mongo Connected.")
}

func (db *DB) GetMongoClient() *mongo.Client {
	return db.client
}

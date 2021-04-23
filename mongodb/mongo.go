package mongodb

import (
	"context"
	"log"

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

func (db *DB) ConnectDB(connectionString string) {
	var err error

	creds := options.Credential{}
	creds.Username = "root"
	creds.Password = "password123"
	clientOptions := options.Client()
	clientOptions.ApplyURI(connectionString)
	clientOptions.SetAuth(creds)

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
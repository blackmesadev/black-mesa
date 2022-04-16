package db

import (
	"context"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson"
)

// functions to handle untrustworthy entries in the database

func AddUntrustworthy(u *structs.Untrustworthy) error {
	col := db.client.Database("black-mesa").Collection("untrustworthy")

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	_, err := col.InsertOne(ctx, u)
	return err

}

func GetSingleUntrustworthy(in *structs.Untrustworthy) (*structs.Untrustworthy, error) {
	var out *structs.Untrustworthy
	col := db.client.Database("black-mesa").Collection("untrustworthy")

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	result := col.FindOne(ctx, in)
	err := result.Decode(&out)
	if out == nil || err != nil {
		return nil, err
	}

	return out, nil
}

func GetMultipleUntrustworthy(in []*structs.Untrustworthy) ([]*structs.Untrustworthy, error) {
	var out []*structs.Untrustworthy
	col := db.client.Database("black-mesa").Collection("untrustworthy")

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	result, err := col.Find(ctx, in)
	if err != nil {
		return nil, err
	}

	err = result.All(ctx, &out)
	if err != nil {
		return nil, err
	}

	return out, nil
}

func GetAllUntrustworthy() ([]structs.Untrustworthy, error) {
	var u []structs.Untrustworthy
	col := db.client.Database("black-mesa").Collection("untrustworthy")

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	result, err := col.Find(ctx, bson.M{})
	if err != nil {
		return nil, err
	}

	err = result.All(ctx, &u)
	if err != nil {
		return nil, err
	}

	return u, nil
}

func DeleteUntrustworthy(id string) error {
	col := db.client.Database("black-mesa").Collection("untrustworthy")

	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"id": id}

	_, err := col.DeleteOne(ctx, filters)
	return err
}

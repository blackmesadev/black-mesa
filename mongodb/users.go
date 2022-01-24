package mongodb

import (
	"context"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"go.mongodb.org/mongo-driver/bson"
)

func (db *DB) GetBlackMesaUser(id string) (*structs.BlackMesaUser, error) {
	var user *structs.BlackMesaUser
	col := db.client.Database("black-mesa").Collection("users")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"id": id}

	result := col.FindOne(ctx, filters)
	err := result.Decode(&user)
	if user == nil || err != nil {
		return nil, err
	}

	return user, nil
}

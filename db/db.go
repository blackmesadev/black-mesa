package db

import (
	"github.com/blackmesadev/black-mesa/structs"
)

var db *DB

func StartDB(cfg structs.MongoConfig) {
	db = InitDB()
	db.ConnectDB(cfg)
}

func GetDB() *DB {
	return db
}

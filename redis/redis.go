package redis

import (
	"log"

	"github.com/go-redis/redis/v8"
)

var r *redis.Client

func ConnectRedis(addr string) *redis.Client {
	var err error

	r = redis.NewClient(&redis.Options{
		Addr: addr,
		DB: 0,
	})

	err = r.Ping(r.Context()).Err()
	if err != nil {
		log.Fatalln("Redis Connection Failed. Unable to start, ", err)
	}

	log.Println("Redis Connected.")

	return r
}

func GetRedis() *redis.Client {
	return r
}
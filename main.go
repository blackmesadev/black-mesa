package main

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/discord"
	"github.com/blackmesadev/black-mesa/redis"
)

func main() {

	config.StartDB()
	redis.ConnectRedis("localhost:6379")

	bot := discord.CreateBot()

	bot.Start()

}

package main

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/discord"
	"github.com/blackmesadev/black-mesa/redis"
)

func main() {

	configFlat := config.LoadFlatConfig()

	config.StartDB(configFlat.Mongo)
	redis.ConnectRedis(configFlat.Redis)

	bot := discord.CreateBot(configFlat.Token)

	bot.Start()

}

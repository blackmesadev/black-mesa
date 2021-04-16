package main

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/discord"
)

func main() {

	config.StartDB()

	bot := discord.CreateBot()

	bot.Start()

}

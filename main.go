package main

import (
	"github.com/trollrocks/black-mesa/config"
	"github.com/trollrocks/black-mesa/discord"
)

func main() {

	config.StartDB()

	bot := discord.CreateBot()

	bot.Start()

}

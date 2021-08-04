package main

import (
	"fmt"
	"io/ioutil"
	"log"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/discord"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/redis"
)

func main() {

	startupMsg, err := ioutil.ReadFile("black-mesa-logo")
	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf("%v\nBlack Mesa Version %v starting\n", string(startupMsg), info.VERSION)

	configFlat := config.LoadFlatConfig()

	config.StartDB(configFlat.Mongo)
	redis.ConnectRedis(configFlat.Redis)
	//music.LavalinkInit(configFlat.Lavalink)

	bot := discord.CreateBot(configFlat.Token)

	bot.Start()

}

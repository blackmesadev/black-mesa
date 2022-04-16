package main

import (
	"fmt"
	"io/ioutil"
	"log"

	"github.com/blackmesadev/black-mesa/apiwrapper"
	"github.com/blackmesadev/black-mesa/db"
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

	configFlat := db.LoadFlatConfig()

	db.StartDB(configFlat.Mongo)
	redis.ConnectRedis(configFlat.Redis)
	apiwrapper.InitAPI(configFlat.API)

	bot := discord.CreateBot(configFlat.Token)

	bot.Start()

}

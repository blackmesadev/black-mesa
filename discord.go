package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/bwmarrin/discordgo"
	"github.com/trollrocks/black-mesa/misc"
)

type Bot struct {
	Session  *discordgo.Session
	Token    string `json:"token"`
	Commands map[string]interface{}
	Prefix   string
}

func startBot() {
	var err error
	bot.Session, err = discordgo.New("Bot " + bot.Token)
	if err != nil {
		log.Fatalln(err)
	}
	initCommandMap()
	initHandlers()
	bot.Session.Identify.Intents = discordgo.MakeIntent(discordgo.IntentsAllWithoutPrivileged)

	bot.Prefix = "!"

	err = bot.Session.Open()
	if err != nil {
		log.Fatalln(err)
		return
	}

	fmt.Printf("Bot started. Press CTRL-C to exit")
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc

	bot.Session.Close()

}

func initCommandMap() {
	commands := make(map[string]interface{})

	commands["help"] = misc.Help

	bot.Commands = commands
}

func initHandlers() {
	//bot.Session.AddHandler()

	// New Message
	bot.Session.AddHandler(messageHandler)
	bot.Session.AddHandler(messageDeleteHandler)
}

package discord

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/blackmesadev/discordgo"
)

var instance *Bot

var Router = NewRouter()

type Bot struct {
	Session  *discordgo.Session
	Token    string `json:"token"`
	Commands map[string]interface{}
	Version  string
	Router   *Mux
}

func CreateBot() *Bot {
	instance = &Bot{}
	instance.getToken()
	instance.Version = "16042021Alpha"

	return instance
}

func GetInstance() *Bot {
	return instance
}

func (bot *Bot) Start() {
	var err error

	bot.Session, err = discordgo.New("Bot " + bot.Token)
	if err != nil {
		log.Fatalln(err)
	}

	// Event listeners
	bot.Session.AddHandler(bot.OnMessageCreate)
	bot.Session.AddHandler(bot.OnMessageUpdate)
	bot.Session.AddHandler(bot.OnMessageDelete)

	bot.Router.InitRouter()

	bot.Session.Identify.Intents = discordgo.MakeIntent(discordgo.IntentsAllWithoutPrivileged)

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

func (bot *Bot) getToken() {
	file, err := os.Open("token.json")
	if err != nil {
		log.Fatalln(err)
	}
	defer func() {
		if err = file.Close(); err != nil {
			log.Fatalln(err)
		}
	}()

	token, err := ioutil.ReadAll(file)
	if err != nil {
		log.Fatalln(err)
	}

	json.Unmarshal(token, bot)
}

package discord

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/music"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

var instance *Bot

type Bot struct {
	Session *discordgo.Session
	Token   string `json:"token"`
	Version string
	Router  *Mux
}

func CreateBot(token string) *Bot {
	instance = &Bot{}
	instance.Token = token
	instance.Version = info.VERSION

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
	bot.Session.AddHandler(bot.OnMemberJoin)
	bot.Session.AddHandler(bot.OnReady)

	bot.Session.AddHandler(music.VoiceUpdate)

	bot.Router = NewRouter()
	bot.Router.InitRouter()

	bot.Session.Identify.Intents = discordgo.MakeIntent(discordgo.IntentsAll)

	bot.Session.State.MaxMessageCount = 25000

	err = bot.Session.Open()
	if err != nil {
		log.Fatalln(err)
		return
	}

	go util.CalcStats(bot.Session)
	go actionExpiryGoroutine()

	log.Println("Black Mesa has finished initalizing successfully.")
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc

	bot.Session.Close()

}

func actionExpiryGoroutine() {
	db := config.GetDB().GetMongoClient().Database("black-mesa").Collection("actions")

	log.Println("action expiry ready")
	for {
		time.Sleep(time.Second)

		timeSec := time.Now().Unix()
		query := bson.M{
			"expires": bson.M{
				"$lte": timeSec,
			},
		}

		cursor, err := db.Find(context.TODO(), query)

		if err != nil {
			log.Println("error whilst dealing with expiring punishments", err)
			continue
		}

		for cursor.Next(context.TODO()) {
			doc := mongodb.Action{}
			cursor.Decode(&doc)
			go func(doc mongodb.Action) {
				fmt.Println(doc)
				switch doc.Type {
				case "ban":
					config.RemoveAction(doc.UUID, doc.UUID)
					GetInstance().Session.GuildBanDelete(doc.GuildID, doc.UserID)
				case "role":
					config.RemoveAction(doc.UUID, doc.UUID)
					GetInstance().Session.GuildMemberRoleRemove(doc.GuildID, doc.UserID, doc.RoleID)
				case "mute":
					config.RemoveAction(doc.UUID, doc.UUID)
					GetInstance().Session.GuildMemberRoleRemove(doc.GuildID, doc.UserID, doc.RoleID)
				case "strike":
					config.RemoveAction(doc.UUID, doc.UUID)
				default:
					fmt.Println("unknown type", doc.Type)
				}
			}(doc)
		}

		cursor.Close(context.TODO())

		db.DeleteMany(context.TODO(), query)
	}
}

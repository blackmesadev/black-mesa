package discord

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/modules/music"
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
	go bot.actionExpiryGoroutine()

	log.Println("Black Mesa has finished initalizing successfully.")
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc

	bot.Session.Close()

}

func (bot *Bot) actionExpiryGoroutine() {
	inst := db.GetDB().GetMongoClient().Database("black-mesa").Collection("actions")

	log.Println("action expiry ready")
	timer := time.NewTicker(time.Second)
	for {
		select {
		case <-timer.C:
			timeSec := time.Now().Unix()
			query := bson.M{
				"expires": bson.M{
					"$lte": timeSec,
				},
			}

			cursor, err := inst.Find(context.TODO(), query)

			if err != nil {
				log.Println("error whilst dealing with expiring punishments", err)
			}

			for cursor.Next(context.TODO()) {
				doc := db.Action{}
				cursor.Decode(&doc)
				go func(doc db.Action) {
					fmt.Println(doc)
					switch doc.Type {
					case "ban":
						db.RemoveAction(doc.UUID, doc.UUID)
						bot.Session.GuildBanDelete(doc.GuildID, doc.UserID)
					case "role":
						db.RemoveAction(doc.UUID, doc.UUID)
						bot.Session.GuildMemberRoleRemove(doc.GuildID, doc.UserID, doc.RoleID)
					case "mute":
						db.RemoveAction(doc.UUID, doc.UUID)
						bot.Session.GuildMemberRoleRemove(doc.GuildID, doc.UserID, doc.RoleID)
					case "strike":
						db.RemoveAction(doc.UUID, doc.UUID)
					default:
						fmt.Println("unknown type", doc.Type)
					}
				}(doc)
			}

			cursor.Close(context.TODO())

			inst.DeleteMany(context.TODO(), query)
		}

	}
}

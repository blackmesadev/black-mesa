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
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

var instance *Bot

type Bot struct {
	Session  *discordgo.Session
	Token    string `json:"token"`
	Commands map[string]interface{}
	Version  string
	Router   *Mux
}

func CreateBot(token string) *Bot {
	instance = &Bot{}
	instance.Token = token
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

	bot.Router = NewRouter()
	bot.Router.InitRouter()

	bot.Session.Identify.Intents = discordgo.MakeIntent(discordgo.IntentsAll)

	bot.Session.State.MaxMessageCount = 5000

	err = bot.Session.Open()
	if err != nil {
		log.Fatalln(err)
		return
	}

	go punishmentExpiryGoroutine()

	fmt.Println("Bot started. Press CTRL-C to exit")
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc

	bot.Session.Close()

}

func punishmentExpiryGoroutine() {
	db := config.GetDB().GetMongoClient().Database("black-mesa").Collection("punishments")

	log.Println("punishment expiry ready")
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
			log.Println("next")
			doc := mongodb.MongoPunishment{}
			cursor.Decode(doc)
			go func() {
				switch doc.PunishmentType {
				case "ban":
					GetInstance().Session.GuildBanDelete(doc.GuildID, doc.UserID)
				case "role":
					GetInstance().Session.GuildMemberRoleRemove(doc.GuildID, doc.UserID, doc.RoleID)
				case "strike":
					// can ignore, strikes don't have anything special about them when they expire
				default:
					fmt.Println("unknown punishment type", doc.PunishmentType)
				}
			}()
		}

		cursor.Close(context.TODO())

		db.DeleteMany(context.TODO(), query)
	}
}

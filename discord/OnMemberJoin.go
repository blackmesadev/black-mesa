package discord

import (
	"context"
	"log"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
)

func (bot *Bot) OnMemberJoin(s *discordgo.Session, m *discordgo.GuildMemberAdd) {
	db := config.GetDB().GetMongoClient().Database("black-mesa").Collection("actions")

	// we only need to check if it exists so we can ignore the actual data since
	// if the driver doesnt find anything it will return a mongo.ErrNoDocuments error type
	_, err := db.Find(context.TODO(), bson.M{
		"guildID": m.GuildID,
		"userID":  m.User.ID,
		"type":    "mute",
	})

	if err == mongo.ErrNoDocuments {
		return
	}

	if err != nil {
		log.Println(err)
		return
	}

	roleid := config.GetMutedRole(m.GuildID, nil)

	err = s.GuildMemberRoleAdd(m.GuildID, m.User.ID, roleid)
	if err != nil {
		log.Println(err)
	}

}

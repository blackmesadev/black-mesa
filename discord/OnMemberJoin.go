package discord

import (
	"context"
	"log"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

func (bot *Bot) OnMemberJoin(s *discordgo.Session, m *discordgo.GuildMemberAdd) {
	db := config.GetDB().GetMongoClient().Database("black-mesa").Collection("actions")

	cur, err := db.Find(context.TODO(), bson.M{
		"guildID": m.GuildID,
		"userID":  m.User.ID,
		"type":    "mute",
	})

	if err != nil {
		log.Println(err)
		return
	}

	var mute *mongodb.Action

	for cur.Next(context.TODO()) {
		mute = &mongodb.Action{}
		cur.Decode(mute)
	}

	// double check that this is a valid mute after decoding
	if mute == nil || mute.Type != "mute" {
		return
	}

	roleid := config.GetMutedRole(m.GuildID, nil)

	err = s.GuildMemberRoleAdd(m.GuildID, m.User.ID, roleid)
	if err != nil {
		log.Println(err)
	}

}

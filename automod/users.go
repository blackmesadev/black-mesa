package automod

import (
	"context"
	"fmt"
	"log"
	"runtime"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

func alertMentionedUsers(s *discordgo.Session, guildID string, mentions []*discordgo.User) error {
	guild, err := s.Guild(guildID)
	if err != nil {
		return err
	}

	for _, m := range mentions {
		s.UserMessageSendEmbed(m.ID, createMentionedEmbed(guild, m))
	}

	return nil
}

func createMentionedEmbed(guild *discordgo.Guild, pingedBy *discordgo.User) *discordgo.MessageEmbed {
	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	fields := []*discordgo.MessageEmbedField{
		{
			Name:   "Server Name",
			Value:  guild.Name,
			Inline: true,
		},
		{
			Name:  "Pinged by",
			Value: pingedBy.String(),
		},
	}

	embed := &discordgo.MessageEmbed{
		URL:         consts.WEBSITE,
		Type:        discordgo.EmbedTypeRich,
		Title:       "You were pinged!",
		Description: "But the message was removed due to violating the servers spam configuration.",
		Color:       0,
		Footer:      footer,
		Fields:      fields,
	}
	return embed
}

// Doesn't need to return anything because this should handle everything silently
func ProcessGuildMemberAdd(s *discordgo.Session, m *discordgo.GuildMemberAdd, conf *structs.Config) {
	if conf.Modules.Automod.GuildOptions == nil {
		return
	}

	minAccAge := util.ParseTime(conf.Modules.Automod.GuildOptions.MinimumAccountAge)

	processMuted(s, m)

	if ok := processDates(s, m, minAccAge); !ok {
		// TODO: do something useful here
		return
	}
}

func processDates(s *discordgo.Session, ma *discordgo.GuildMemberAdd, maxDifference int64) bool {
	m := ma.Member

	difference := int64(m.JoinedAt.Sub(util.SnowflakeToTimestamp(m.User.ID)))

	return difference <= maxDifference

}

func processMuted(s *discordgo.Session, m *discordgo.GuildMemberAdd) {
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

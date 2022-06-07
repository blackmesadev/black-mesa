package automod

import (
	"fmt"
	"log"
	"runtime"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/mongo"
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
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
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
func ProcessGuildMemberAdd(s *discordgo.Session, ma *discordgo.GuildMemberAdd, conf *structs.Config) {
	processMuted(s, ma, conf)

	if conf.Modules.Automod.GuildOptions != nil {
		return
	}

	minAccAge := util.ParseTime(conf.Modules.Automod.GuildOptions.MinimumAccountAge)

	if ok := processDates(s, ma.Member, minAccAge); !ok {
		// TODO: do something useful here
		return
	}
}

func processDates(s *discordgo.Session, m *discordgo.Member, maxDifference int64) bool {
	difference := int64(m.JoinedAt.Sub(util.SnowflakeToTimestamp(m.User.ID)))

	return difference <= maxDifference

}

func processMuted(s *discordgo.Session, ma *discordgo.GuildMemberAdd, conf *structs.Config) {

	mute, err := db.GetMute(ma.GuildID, ma.User.ID)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			return
		}
		mute = nil
	}

	if mute != nil {
		err = s.GuildMemberRoleAdd(ma.GuildID, ma.User.ID, conf.Modules.Moderation.MuteRole)
		if err != nil {
			log.Println(err)
		}
	}
}

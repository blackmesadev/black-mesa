package automod

import (
	"fmt"
	"runtime"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/discordgo"
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

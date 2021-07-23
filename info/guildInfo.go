package info

import (
	"fmt"
	"log"
	"runtime"
	"strconv"

	"github.com/blackmesadev/discordgo"
)

func GuildInfoCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", VERSION, runtime.Version()),
	}

	guild, err := s.Guild(m.GuildID)
	if err != nil {
		log.Println(err)
		return
	}

	fields := []*discordgo.MessageEmbedField{
		{
			Name:   "ID",
			Value:  guild.ID,
			Inline: true,
		},
		{
			Name:   "Name",
			Value:  guild.Name,
			Inline: true,
		},
		{
			Name:   "Region",
			Value:  guild.Region,
			Inline: true,
		},
		{
			Name:   "Owner",
			Value:  fmt.Sprintf("<@%v>", guild.OwnerID),
			Inline: true,
		},
		{
			Name:   "Member Count",
			Value:  strconv.Itoa(guild.ApproximateMemberCount),
			Inline: true,
		},
		{
			Name:   "Max Members",
			Value:  strconv.Itoa(guild.MaxMembers),
			Inline: true,
		},
		{
			Name:   "Role Count",
			Value:  strconv.Itoa(len(guild.Roles)),
			Inline: true,
		},
		{
			Name:   "Emoji Count",
			Value:  strconv.Itoa(len(guild.Emojis)),
			Inline: true,
		},
		{
			Name:   "Channel Count",
			Value:  strconv.Itoa(len(guild.Channels)),
			Inline: true,
		},
		{
			Name:   "Vanity URL",
			Value:  strconv.Itoa(len(guild.Channels)),
			Inline: true,
		},
	}

	thumbnail := &discordgo.MessageEmbedThumbnail{
		URL:    guild.IconURL(),
		Width:  200,
		Height: 200,
	}

	embed := &discordgo.MessageEmbed{
		URL:       WEBSITE,
		Type:      discordgo.EmbedTypeRich,
		Title:     fmt.Sprintf("%v Guild Info", guild.Name),
		Color:     0,
		Thumbnail: thumbnail,
		Footer:    footer,
		Fields:    fields,
	}

	s.ChannelMessageSendEmbed(m.ChannelID, embed)
}

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

	var memberCount string
	stateGuild, err := s.State.Guild(m.GuildID)
	if err != nil || stateGuild == nil {
		memberCount = strconv.Itoa(guild.ApproximateMemberCount)
	} else {
		memberCount = strconv.Itoa(stateGuild.MemberCount)
	}

	var invite string
	invites, err := s.GuildInvites(m.GuildID)
	if err != nil {
		invite = ""
	} else {
		var prevMax int
		for _, v := range invites {
			if v.MaxUses > prevMax {
				invite = v.Code
				prevMax = v.Uses
			}
		}
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
			Value:  memberCount,
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
	}

	if guild.VanityURLCode != "" {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Vanity URL",
			Value:  guild.VanityURLCode,
			Inline: true,
		})
		invite = fmt.Sprintf("https://discord.gg/%v", guild.VanityURLCode)

	}

	thumbnail := &discordgo.MessageEmbedThumbnail{
		URL:    guild.IconURL(),
		Width:  200,
		Height: 200,
	}

	embed := &discordgo.MessageEmbed{
		URL:       invite,
		Type:      discordgo.EmbedTypeRich,
		Title:     fmt.Sprintf("%v Guild Info", guild.Name),
		Color:     0,
		Thumbnail: thumbnail,
		Footer:    footer,
		Fields:    fields,
	}

	s.ChannelMessageSendEmbed(m.ChannelID, embed)
}

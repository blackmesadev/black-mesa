package info

import (
	"fmt"
	"runtime"
	"strconv"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

var failureMsg = fmt.Sprintf("%v Unable to fetch Guild data.", consts.EMOJI_CROSS)

func GuildInfoCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	start := time.Now()

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", VERSION, runtime.Version()),
	}

	guild, err := s.State.Guild(m.GuildID)
	if err == discordgo.ErrStateNotFound || guild == nil {
		guild, err = s.Guild(m.GuildID)
		if err != nil || guild == nil {
			s.ChannelMessageSend(m.ChannelID, failureMsg)
			return
		}
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
	if err != nil || len(invites) == 0 {
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

	var createdUnix int
	snowflakeInt, err := strconv.Atoi(guild.ID)
	if err != nil {
		createdUnix = 0
	}

	createdUnix = snowflakeInt>>22 + 1420070400000 // bitsift and add discord epoch for unix timestamp

	timestamp := time.UnixMilli(int64(createdUnix))

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
			Name:   "Created",
			Value:  timestamp.Format(time.RFC3339),
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
	}

	if stateGuild != nil {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Channel Count",
			Value:  strconv.Itoa(len(stateGuild.Channels)),
			Inline: true,
		})
	}

	if guild.VanityURLCode != "" {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Vanity URL",
			Value:  guild.VanityURLCode,
			Inline: true,
		})
		invite = fmt.Sprintf("https://discord.gg/%v", guild.VanityURLCode)

	}

	if guild.PreferredLocale != "" {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Locale",
			Value:  guild.PreferredLocale,
			Inline: true,
		})
	}

	thumbnail := &discordgo.MessageEmbedThumbnail{
		URL:    guild.IconURL(),
		Width:  256,
		Height: 256,
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

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

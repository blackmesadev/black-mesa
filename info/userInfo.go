package info

import (
	"fmt"
	"log"
	"runtime"
	"time"

	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func UserInfoCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	start := time.Now()

	var userId string

	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	userId = m.Author.ID

	if len(idList) > 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `userinfo <target:user>`")
		return
	}

	if len(idList) == 1 {
		userId = idList[0]
	} else {
		userId = m.Author.ID
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", VERSION, runtime.Version()),
	}

	member, err := s.State.Member(m.GuildID, userId)
	if err == discordgo.ErrStateNotFound || member == nil {
		member, err = s.GuildMember(m.GuildID, userId)
		if err != nil || member == nil {
			s.ChannelMessageSend(m.ChannelID, failureMsg)
			return
		}
	}

	roleList := member.Roles
	guildRoles, err := s.GuildRoles(m.GuildID)
	if err != nil {
		log.Println(err)
		return
	}

	var highestRole *discordgo.Role
	var highestRolePos int

	for _, role := range guildRoles {
		for _, userRole := range roleList {
			if role.ID == userRole && highestRolePos < role.Position {
				highestRole = role
				highestRolePos = role.Position
				break
			}
		}
	}

	fields := []*discordgo.MessageEmbedField{
		{
			Name:   "ID",
			Value:  fmt.Sprintf("`%v`", member.User.ID),
			Inline: true,
		},
		{
			Name:   "Joined",
			Value:  fmt.Sprintf("`%v`", member.JoinedAt),
			Inline: true,
		},
		{
			Name:  "Top Role",
			Value: fmt.Sprintf("`%v`", highestRole.Name),
		},
	}

	if member.Nick != "" {
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:   "Nickname",
			Value:  fmt.Sprintf("`%v`", member.Nick),
			Inline: true,
		})
	}

	thumbnail := &discordgo.MessageEmbedThumbnail{
		URL:    member.User.AvatarURL("256"),
		Width:  256,
		Height: 256,
	}

	embed := &discordgo.MessageEmbed{
		Type:      discordgo.EmbedTypeRich,
		Title:     fmt.Sprintf("%v's User Info", member.User.String()),
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

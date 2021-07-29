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

	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 || len(idList) > 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `userinfo <target:user>`")
		return
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", VERSION, runtime.Version()),
	}

	member, err := s.GuildMember(m.GuildID, idList[0])
	if err != nil {
		log.Println(err)
		return
	}

	roleList := member.Roles
	guildRoles, err := s.GuildRoles(m.GuildID)
	if err != nil {
		log.Println(err)
		return
	}

	var highestRole *discordgo.Role
	highestRolePos := len(guildRoles)

	for _, role := range guildRoles {
		for _, userRole := range roleList {
			if role.ID == userRole && highestRolePos > role.Position {
				highestRole = role
				highestRolePos = role.Position
				break
			}
		}
	}

	fields := []*discordgo.MessageEmbedField{
		{
			Name:   "ID",
			Value:  member.User.ID,
			Inline: true,
		},
		{
			Name:   "Joined",
			Value:  fmt.Sprintf("`%v`", member.JoinedAt),
			Inline: true,
		},
		{
			Name:   "Nickname",
			Value:  member.Nick,
			Inline: true,
		},
		{
			Name:  "Top Role",
			Value: fmt.Sprintf("`%v`", highestRole.Name),
		},
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

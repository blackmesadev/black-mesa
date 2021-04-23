package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

func MuteCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	var reason string

	start := time.Now()

	idList := snowflakeRegex.FindAllString(m.Content, -1)

	reasonSearch := snowflakeRegex.Split(m.Content, -1)

	if reasonSearch[len(reasonSearch)-1][:1] == ">" {
		reason = reasonSearch[len(reasonSearch)-1][1:]
	} else {
		reason = reasonSearch[len(reasonSearch)-1]
	}

	reason = strings.TrimSpace(reason)

	roleid := config.GetMutedRole(m.GuildID)
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	msg := "Successfully muted "

	unableMute := make([]string, 0)
	for _, id := range idList {

		err := s.GuildMemberRoleAdd(m.GuildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableMute = append(unableMute, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableMute) != 0 {
		msg += fmt.Sprintf("\nCould not mute %v", unableMute)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))

}

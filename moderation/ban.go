package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/discordgo"
)

func BanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	var permBan bool
	var reason string
	var duration time.Duration

	start := time.Now()

	idList := snowflakeRegex.FindAllString(m.Content, -1)
	params := snowflakeRegex.Split(m.Content, -1)

	reason = params[len(params)-1]
	durationString := params[len(params)-2]

	if strings.Contains(durationString, "ban") {
		permBan = true
	}

	duration, err := time.ParseDuration(durationString)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, err.Error())
		permBan = true
	}

	msg := "Successfully banned "

	unableBan := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
		if err != nil {
			unableBan = append(unableBan, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
		}
	}

	if permBan {
		msg += fmt.Sprintf("for reason `%v` lasting `Forever`.", reason)

	} else {
		msg += fmt.Sprintf("for reason `%v` lasting `%v`.", reason, duration.String())
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\nCould not ban %v", unableBan)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
}

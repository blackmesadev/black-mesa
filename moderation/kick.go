package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/discordgo"
)

func KickCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
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

	msg := "<:mesaKick:832350526778900571> Successfully kicked "

	unableKick := make([]string, 0)
	for _, id := range idList {
		err := s.GuildMemberDeleteWithReason(m.GuildID, id, reason)
		if err != nil {
			unableKick = append(unableKick, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableKick) != 0 {
		msg += fmt.Sprintf("\nCould not kick %v", unableKick)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))

}

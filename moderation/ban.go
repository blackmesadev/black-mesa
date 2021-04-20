package moderation

import (
	"fmt"
	"time"

	"github.com/blackmesadev/discordgo"
)

func BanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {

	start := time.Now()

	var permBan bool

	idList, duration, reason := parseCommand(m.Content)

	if duration == 0 {
		permBan = true
	}

	parse := time.Since(start)

	msg := "Successfully banned "

	dstart := time.Now()
	unableBan := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
		if err != nil {
			unableBan = append(unableBan, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
		}
	}
	discord := time.Since(dstart)
	msgs := time.Now()
	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if permBan {
		msg += "lasting `Forever`."

	} else {
		msg += fmt.Sprintf("expiring `%v`.", time.Unix(duration, 0))
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\nCould not ban %v", unableBan)
	}

	msgsTotal := time.Since(msgs)
	s.ChannelMessageSend(m.ChannelID, msg)

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v (%v parsing, %v discordapi, %v message creation)",
		time.Since(start), parse, discord, msgsTotal))
}

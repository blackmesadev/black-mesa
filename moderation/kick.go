package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func KickCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	var reason string

	start := time.Now()

	idList := snowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `kick <target:user[]> [reason:string...]`")
		return
	}

	reasonSearch := snowflakeRegex.Split(m.Content, -1)

	if reasonSearch[len(reasonSearch)-1][:1] == ">" {
		reason = reasonSearch[len(reasonSearch)-1][1:]
	} else {
		reason = reasonSearch[len(reasonSearch)-1]
	}

	reason = strings.TrimSpace(reason)

	msg := "<:mesaCheck:832350526729224243> Successfully kicked "

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
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not kick %v", unableKick)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

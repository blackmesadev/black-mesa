package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func UnmuteCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.mute") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

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

	msg := "Successfully unmuted "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableUnmute := make([]string, 0)
	for _, id := range idList {

		err := s.GuildMemberRoleRemove(m.GuildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableUnmute = append(unableUnmute, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)

			possibleUser, err := s.State.Member(m.GuildID, id)
			if err != nil { continue }
			logging.LogUnmute(s, m.GuildID, fullName, possibleUser.User, reason)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableUnmute) != 0 {
		msg += fmt.Sprintf("\nCould not unmute %v", unableUnmute)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

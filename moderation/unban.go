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

func UnbanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.ban") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	var reason string

	start := time.Now()

	idList := snowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `unban <target:user[]> [reason:string...]`")
		return
	}

	reasonSearch := snowflakeRegex.Split(m.Content, -1)

	search := reasonSearch[len(reasonSearch)-1]

	if search != "" {
		if search[:1] == ">" {
			reason = reasonSearch[len(reasonSearch)-1][1:]
		} else {
			reason = reasonSearch[len(reasonSearch)-1]
		}
	}

	reason = strings.TrimSpace(reason)

	msg := "<:mesaCheck:832350526729224243> Successfully unbanned "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableUnban := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanDeleteWithReason(m.GuildID, id, reason)
		if err != nil {
			unableUnban = append(unableUnban, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)

			user := fmt.Sprintf("`%v`", id)
			possibleUser, err := s.State.Member(m.GuildID, id)
			if err == nil {
				user = fmt.Sprintf("%v#%v (`%v`)", possibleUser.User.Username, possibleUser.User.Discriminator, possibleUser.User.ID)
			}

			logging.LogUnban(s, m.GuildID, fullName, user, reason)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableUnban) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not unban %v", unableUnban)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func SoftBanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.softban") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	var reason string

	start := time.Now()

	idList := snowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `softban <target:user[]> [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
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

	msg := "<:mesaCheck:832350526729224243> Successfully softbanned "

	unableBan := make([]string, 0)
	unableUnban := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanCreateWithReason(m.GuildID, id, reason, 1) // todo: make the days configurable via cmd params + config (default setting)
		if err != nil {
			unableBan = append(unableBan, id)
		} else {
			err := s.GuildBanDeleteWithReason(m.GuildID, id, "Softban")
			if err != nil && unableBan[len(unableBan)-1] != id { // make sure that the person just wasnt banned in the first place aswell
				unableUnban = append(unableUnban, id)
			}
			msg += fmt.Sprintf("<@%v> ", id)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not softban %v", unableBan)
	}

	if len(unableUnban) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not unban %v", unableUnban)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

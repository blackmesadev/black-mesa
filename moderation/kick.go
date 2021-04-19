package moderation

import (
	"fmt"
	"time"

	"github.com/blackmesadev/discordgo"
)

func KickCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	start := time.Now()

	idList := snowflakeRegex.FindAllString(m.Content, -1)

	msg := "Successfully removed "

	unableBan := make([]string, 0)
	for _, id := range idList {
		err := s.GuildMemberDelete(m.GuildID, id)
		if err != nil {
			unableBan = append(unableBan, id)
		} else {
			msg += fmt.Sprintf("<@%v>", id)
		}
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\nCould not remove %v", unableBan)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))

}

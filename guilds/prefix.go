package guilds

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func PrefixCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	start := time.Now()

	prefix := config.GetPrefix(m.GuildID)

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("The Prefix for this Guild is `%v`", prefix))

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

package guilds

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func PrefixCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	start := time.Now()

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("The Prefix for this Guild is `%v`", conf.Prefix))

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

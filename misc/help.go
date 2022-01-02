package misc

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func HelpCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	msg := fmt.Sprintf("Help can be found at %v", consts.WEBSITE)
	s.ChannelMessageSend(m.ChannelID, msg)
}

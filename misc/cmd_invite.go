package misc

import (
	"github.com/blackmesadev/discordgo"
)

const botInvite string = "The Official Bot can be invited at "

func InviteCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	s.ChannelMessageSend(m.ChannelID, botInvite)
}

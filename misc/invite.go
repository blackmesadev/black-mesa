package misc

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

var inviteMsg string = fmt.Sprintf("<:blackmesa:834786866413830185> Black Mesa can be invited at %v\nYou can join the Discord at %v", consts.WEBSITE, consts.DISCORDINVITE)

func InviteCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	s.ChannelMessageSend(m.ChannelID, inviteMsg)
}

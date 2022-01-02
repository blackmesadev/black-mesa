package util

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func NoPermissionHandler(s *discordgo.Session, m *discordgo.Message, conf *structs.Config, permission string) {
	if config.GetDisplayNoPermission(m.GuildID, conf) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCross:832350526414127195> You do not have permission to `%v`.", permission))
	}
}

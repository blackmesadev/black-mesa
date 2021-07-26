package moderation

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

func PurgeCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, PERMISSION_PURGE) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}
}

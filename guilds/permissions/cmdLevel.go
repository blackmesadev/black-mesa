package permissions

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/discordgo"
)

func GetCommandLevelCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_VIEWPERMS) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}
	msg := "```\nCommand Permission Levels:\n"
	for _, cmd := range args {
		lvl := config.GetLevel(s, m.GuildID, cmd)
		msg = fmt.Sprintf("%v%v:`%d`\n", msg, cmd, lvl)
	}
	msg += "```"

	s.ChannelMessageSend(m.ChannelID, msg)
}

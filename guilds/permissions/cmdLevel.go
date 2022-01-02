package permissions

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func GetCommandLevelCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_VIEWPERMS) {
		config.NoPermissionHandler(s, m, conf, consts.PERMISSION_VIEWPERMS)
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

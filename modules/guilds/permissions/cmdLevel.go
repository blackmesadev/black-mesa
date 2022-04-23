package permissions

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func GetCommandLevelCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_VIEWCMDLEVEL)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	msg := "```\nCommand Permission Levels:\n"
	for _, cmd := range args {
		lvl := db.GetLevel(s, conf, m.GuildID, cmd)
		msg = fmt.Sprintf("%v%v:`%d`\n", msg, cmd, lvl)
	}
	msg += "```"

	s.ChannelMessageSend(m.ChannelID, msg)
}

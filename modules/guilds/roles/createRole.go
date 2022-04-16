package roles

import (
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func CreateRoleCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_ROLECREATE) {
		db.NoPermissionHandler(s, m, conf, consts.PERMISSION_ROLECREATE)
		return
	}
}

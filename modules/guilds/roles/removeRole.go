package roles

import (
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func RemoveRoleCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_ROLEREMOVE)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
}

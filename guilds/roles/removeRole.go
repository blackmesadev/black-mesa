package roles

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func RemoveRoleCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_ROLEREMOVE) {
		config.NoPermissionHandler(s, m, conf, consts.PERMISSION_ROLEREMOVE)
		return
	}
}

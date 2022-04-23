package permissions

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func GetUserLevelCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		idList = append(idList, m.Author.ID)
	}

	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_VIEWUSERLEVEL)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	if len(idList) > 10 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `getuserlevel` takes a maximum of 10 `<target:user[]>` parameters.")
		return
	}

	msg := "```\nPermission Levels:\n"

	for _, id := range idList {
		var memberName string

		lvl := db.GetLevel(s, conf, m.GuildID, id)

		member, err := s.State.Member(m.GuildID, id)
		if err != nil {
			memberName = id
		} else {
			memberName = member.User.String()
		}
		msg = fmt.Sprintf("%v%v:`%d`\n", msg, memberName, lvl)
	}
	msg += "```"

	s.ChannelMessageSend(m.ChannelID, msg)
}

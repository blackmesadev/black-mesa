package permissions

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func GetUserLevelCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		idList = append(idList, m.Author.ID)
	}

	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_VIEWPERMS) && idList[0] != m.Author.ID {
		config.NoPermissionHandler(s, m, conf, consts.PERMISSION_VIEWPERMS)
		return
	}

	if len(idList) > 10 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `getuserlevel` takes a maximum of 10 `<target:user[]>` parameters.")
		return
	}

	msg := "```\nPermission Levels:\n"

	for _, id := range idList {
		var memberName string

		lvl := config.GetLevel(s, m.GuildID, id)

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

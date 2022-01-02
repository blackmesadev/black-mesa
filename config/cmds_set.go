package config

import (
	"fmt"
	"strings"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func SetConfigCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	allowed := CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_CONFIGSET)
	if !allowed {
		NoPermissionHandler(s, m, conf, consts.PERMISSION_CONFIGSET)
		return
	}

	if len(ctx.Fields) == 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You must specify a key.")
		return
	}

	if len(ctx.Fields) == 2 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You must specify a value.")
		return
	}

	var combinedValue string
	for i, v := range ctx.Fields {
		if i >= 2 {
			combinedValue = fmt.Sprintf("%v %v", combinedValue, v)
		}
	}

	updates, err := db.SetConfigOne(m.GuildID, ctx.Fields[1], strings.TrimSpace(combinedValue))
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCross:832350526414127195> Failed for reason %v", err))
		return
	}

	if updates.ModifiedCount == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Invalid Key or Value.")
	} else {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCheck:832350526729224243> Updated %v records.", updates.MatchedCount))
	}

}

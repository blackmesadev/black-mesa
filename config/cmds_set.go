package config

import (
	"fmt"

	"github.com/blackmesadev/discordgo"
)

func SetConfigCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	allowed := CheckPermission(s, m.GuildID, m.Member.User.ID, "config.set")
	if !allowed {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission to `config.set`.")
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

	updates, err := db.SetConfigOne(m.GuildID, ctx.Fields[1], combinedValue)
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

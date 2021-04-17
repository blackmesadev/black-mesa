package config

import (
	"encoding/json"
	"fmt"
	"log"
	"strings"

	"github.com/blackmesadev/discordgo"
)

func GetConfigCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	allowed := CheckPermission(s, m.GuildID, m.Member.User.ID, "config.get")
	if !allowed {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission to `config.get`.")
		return
	}

	if len(ctx.Fields) == 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You must specify a key.")
		return
	}

	projections := make([]string, 0, len(ctx.Fields))

	for i, v := range ctx.Fields {
		if i != 0 {
			projections = append(projections, v)
		}
	}

	data, err := db.GetConfigMultipleProjection(m.GuildID, projections)

	delete(*data, "_id")

	if err != nil {
		log.Println(err)
		return
	}

	if data == nil {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> No data found.")
		log.Println(data, projections)
		return
	}

	msg := fmt.Sprintf("<:mesaCommand:832350527131746344> Retrieved %v.", projections)

	dataBytes, err := json.MarshalIndent(data, "", "\t")
	if err != nil {
		log.Println(err)
		return
	}

	jsonString := string(dataBytes)

	s.ChannelFileSendWithMessage(m.ChannelID, msg, "config.json", strings.NewReader(jsonString))
	s.ChannelMessageSend(m.ChannelID, "```json\n"+jsonString+"```")

}

package config

import (
	"encoding/json"
	"log"
	"strings"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func SetupCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	allowed := CheckPermission(s, m.GuildID, m.Author.ID, "admin.setup")
	if !allowed {
		NoPermissionHandler(s, m, conf, "admin.setup")
		return
	}
	g, err := s.Guild(m.GuildID)
	if err != nil {
		log.Println(err)
	}
	conf = AddGuild(g, m.Author.ID)

	bytes, err := json.Marshal(&conf)
	if err != nil {
		log.Println(err)
	}

	s.ChannelFileSend(m.ChannelID, "config.json", strings.NewReader(string(bytes)))
}

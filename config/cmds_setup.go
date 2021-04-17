package config

import (
	"encoding/json"
	"log"
	"strings"

	"github.com/blackmesadev/discordgo"
)

func SetupCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context) {
	g, err := s.Guild(m.GuildID)
	if err != nil {
		log.Println(err)
	}
	conf := AddGuild(g, m.Author.ID)

	bytes, err := json.Marshal(&conf)
	if err != nil {
		log.Println(err)
	}

	s.ChannelFileSend(m.ChannelID, "config.json", strings.NewReader(string(bytes)))
}

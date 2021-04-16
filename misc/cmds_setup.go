package misc

import (
	"encoding/json"
	"log"
	"strings"

	"github.com/bwmarrin/discordgo"
	"github.com/trollrocks/black-mesa/config"
)

func Setup(s *discordgo.Session, m *discordgo.MessageCreate, parameters []string) {
	g, err := s.Guild(m.GuildID)
	if err != nil {
		log.Println(err)
	}
	conf := config.AddConfig(g, m.Author.ID)

	bytes, err := json.Marshal(&conf)
	if err != nil {
		log.Println(err)
	}

	s.ChannelFileSend(m.ChannelID, "config.json", strings.NewReader(string(bytes)))
}

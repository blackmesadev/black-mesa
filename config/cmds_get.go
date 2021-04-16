package config

import (
	"log"

	"github.com/blackmesadev/discordgo"
)

func GetConfigCmd(s *discordgo.Session, m *discordgo.MessageCreate, parameters []string) {
	projection := parameters[1]
	data := getOne(m.GuildID, projection)
	if data != nil {
		log.Println("error occured during GetCmd")
	}

}

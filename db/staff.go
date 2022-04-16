package db

import (
	"log"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func IsStaff(s *discordgo.Session, conf *structs.Config, guildid string, userid string) bool {
	if conf == nil {
		var err error
		conf, err = GetConfig(guildid)
		if err != nil {
			log.Printf("Failed to get config for %v (%v)\n", guildid, err)
			return false
		}
	}

	lvl := GetLevel(s, conf, guildid, userid)

	if lvl >= conf.Modules.Guild.StaffLevel {
		return true
	}

	return false
}

package config

import (
	"log"

	"github.com/bwmarrin/discordgo"
	"github.com/trollrocks/black-mesa/mongodb"
	"github.com/trollrocks/black-mesa/structs"
)

var db *mongodb.DB

func StartDB() {
	db = mongodb.InitDB()
	db.ConnectDB("mongodb://localhost:27017")
}

func AddConfig(g *discordgo.Guild, invokedByUserID string) *structs.Config {
	config := MakeConfig(g, invokedByUserID)
	db.AddConfig(config)
	return config
}

func GetConfig(guildid string) *structs.Config {
	config, err := db.GetConfig(guildid)
	if err != nil {
		log.Println(err)
	}
	return config
}

func GetLevel(c *structs.Config, s *discordgo.Session, guildid string, userid string) int64 {

	// first try userids only
	for k, v := range c.Levels {
		if k == userid {
			return v
		}
	}

	// get roles instead then

	m, err := s.GuildMember(guildid, userid)
	if err != nil {
		log.Println(err)
	}

	var highestLevel int64
	highestLevel = 0

	for _, role := range m.Roles {
		level, ok := c.Levels[role]
		if ok {
			if level > highestLevel {
				highestLevel = level
			}
		}
	}

	return highestLevel
}

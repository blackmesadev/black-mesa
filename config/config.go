package config

import (
	"encoding/json"
	"log"

	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

var db *mongodb.DB

func StartDB() {
	db = mongodb.InitDB()
	db.ConnectDB("mongodb://localhost:27017")
}

func AddGuild(g *discordgo.Guild, invokedByUserID string) *structs.Config {
	config := MakeConfig(g, invokedByUserID)

	db.AddConfig(&mongodb.MongoGuild{
		GuildID: g.ID,
		Config:  config,
	})
	return config
}

func GetConfig(guildid string) *structs.Config {
	config, err := db.GetConfig(guildid)
	if err != nil {
		log.Println(err)
	}
	return config
}

func getOne(guildid string, query string) *bson.M {
	data, err := db.GetConfigProjection(guildid, query)
	if err != nil {
		log.Println(err)
		return nil
	}
	return data
}

func GetPrefix(guildid string) string {
	var prefix string

	data, err := db.GetConfigProjection(guildid, "commands.prefix")
	if err != nil {
		log.Println(err)
		return "!"
	}

	binData, err := json.Marshal(data)
	if err != nil {
		log.Println(err)
		return "!"
	}

	json.Unmarshal(binData, &prefix)

	return prefix

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

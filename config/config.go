package config

import (
	"encoding/json"
	"log"

	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

var db *mongodb.DB

func StartDB() {
	db = mongodb.InitDB()
	db.ConnectDB("mongodb://localhost:27017")
}

func GetDB() *mongodb.DB {
	return db
}

func AddGuild(g *discordgo.Guild, invokedByUserID string) *structs.Config {
	config := MakeConfig(g, invokedByUserID)

	db.AddConfig(&mongodb.MongoGuild{
		GuildID: g.ID,
		Config:  config,
	})
	return config
}

func GetConfig(guildid string) (*structs.Config, error) {
	config, err := db.GetConfig(guildid)
	if err != nil {
		log.Println(err)
		return nil, err
	}
	return config, nil
}

func GetPrefix(guildid string) string {
	prefixMap := make(map[string]map[string]string)

	data, err := db.GetConfigProjection(guildid, "prefix")
	if err != nil {
		log.Println(err)
		return "!"
	}

	binData, err := json.Marshal(data)
	if err != nil {
		log.Println(err)
		return "!"
	}

	json.Unmarshal(binData, &prefixMap)

	return prefixMap["config"]["prefix"]

}

func GetMutedRole(guildid string) string {
	prefixMap := make(map[string]map[string]string)

	data, err := db.GetConfigProjection(guildid, "muteRole")
	if err != nil {
		log.Println(err)
		return ""
	}

	binData, err := json.Marshal(data)
	if err != nil {
		log.Println(err)
		return ""
	}

	json.Unmarshal(binData, &prefixMap)

	return prefixMap["moderation"]["muteRole"]

}

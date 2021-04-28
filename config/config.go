package config

import (
	"encoding/json"
	"io/ioutil"
	"log"
	"os"

	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

var db *mongodb.DB

func LoadFlatConfig() structs.FlatConfig {
	jsonFile, err := os.Open("config.json")

	if err != nil {
		panic(err)
	}

	defer jsonFile.Close()

	bytes, _ := ioutil.ReadAll(jsonFile)

	var result structs.FlatConfig
	json.Unmarshal([]byte(bytes), &result)

	return result
}

func StartDB(cfg structs.MongoConfig) {
	db = mongodb.InitDB()
	db.ConnectDB(cfg)
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
	tempStruct := &mongodb.MongoGuild{}

	data, err := db.GetConfigProjection(guildid, "prefix")
	if err != nil || len(data) == 0 {
		log.Println(err)
		return "!"
	}

	binData, err := json.Marshal(data)
	if err != nil {
		log.Println(err)
		return "!"
	}

	json.Unmarshal(binData, &tempStruct)

	return tempStruct.Config.Prefix

}

func GetMutedRole(guildid string) string {
	tempStruct := &mongodb.MongoGuild{}

	data, err := db.GetConfigProjection(guildid, "modules.moderation.muteRole")
	if err != nil || len(data) == 0 {
		log.Println(err)
		return ""
	}

	binData, err := json.Marshal(data)
	if err != nil {
		log.Println(err)
		return ""
	}

	json.Unmarshal(binData, &tempStruct)

	return tempStruct.Config.Modules.Moderation.MuteRole

}

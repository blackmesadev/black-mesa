package config

import (
	"encoding/json"
	"errors"
	"log"
	"os"

	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/mongo"
)

var db *mongodb.DB

func LoadFlatConfig() structs.FlatConfig {
	mongo := structs.MongoConfig{
		ConnectionString: os.Getenv("MONGOURI"),
		Username:         os.Getenv("MONGOUSER"),
		Password:         os.Getenv("MONGOPASS"),
	}

	redis := structs.RedisConfig{
		Host: os.Getenv("REDIS"),
	}

	api := structs.APIConfig{
		Host:  os.Getenv("APIHOST"),
		Port:  os.Getenv("APIPORT"),
		Token: os.Getenv("APITOKEN"),
	}

	return structs.FlatConfig{
		Token: os.Getenv("TOKEN"),
		Mongo: mongo,
		Redis: redis,
		API:   api,
	}
}

func LoadLavalinkConfig() structs.LavalinkConfig {
	return structs.LavalinkConfig{
		Host:     os.Getenv("LAVALINKURI"),
		Password: os.Getenv("LAVALINKPASS"),
	}
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
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	if config == nil {
		err = errors.New("config is nil")
	}

	return config, err
}

// Takes an optional config parameter incase there's a config struct to get it from already.
func GetPrefix(guildid string, config *structs.Config) string {
	if config == nil || config.Prefix == "" {
		tempStruct := &mongodb.MongoGuild{}

		data, err := db.GetConfigProjection(guildid, "prefix")
		if err != nil || len(data) == 0 {
			return "!"
		}

		binData, err := json.Marshal(data)
		if err != nil {
			return "!"
		}

		json.Unmarshal(binData, &tempStruct)

		return tempStruct.Config.Prefix
	} else {
		return config.Prefix
	}

}

// Takes an optional config parameter incase there's a config struct to get it from already.
func GetMutedRole(guildid string, config *structs.Config) string {
	if config == nil || config.Modules.Moderation == nil {
		tempStruct := &mongodb.MongoGuild{}

		data, err := db.GetConfigProjection(guildid, "modules.moderation.muteRole")
		if err != nil || len(data) == 0 {
			return ""
		}

		binData, err := json.Marshal(data)
		if err != nil {
			return ""
		}

		json.Unmarshal(binData, &tempStruct)

		return tempStruct.Config.Modules.Moderation.MuteRole

	} else {
		return config.Modules.Moderation.MuteRole
	}
}

// Takes an optional config parameter incase there's a config struct to get it from already.
func GetRemoveRolesOnMute(guildid string, config *structs.Config) bool {
	if config == nil || config.Modules.Moderation == nil {
		tempStruct := &mongodb.MongoGuild{}

		data, err := db.GetConfigProjection(guildid, "modules.moderation.removeRolesOnMute")
		if err != nil || len(data) == 0 {
			return false
		}

		binData, err := json.Marshal(data)
		if err != nil {
			return false
		}

		json.Unmarshal(binData, &tempStruct)

		return tempStruct.Config.Modules.Moderation.RemoveRolesOnMute

	} else {
		return config.Modules.Moderation.RemoveRolesOnMute
	}

}

// Takes an optional config parameter incase there's a config struct to get it from already.
func GetDisplayNoPermission(guildid string, config *structs.Config) bool {
	if config == nil || config.Modules.Moderation == nil {
		tempStruct := &mongodb.MongoGuild{}

		data, err := db.GetConfigProjection(guildid, "modules.moderation.displayNoPermission")
		if err != nil || len(data) == 0 {
			return false
		}

		binData, err := json.Marshal(data)
		if err != nil {
			return false
		}

		json.Unmarshal(binData, &tempStruct)

		return tempStruct.Config.Modules.Moderation.DisplayNoPermission

	} else {
		return config.Modules.Moderation.DisplayNoPermission
	}

}

package db

import (
	"errors"
	"log"
	"os"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/mongo"
	"gopkg.in/yaml.v3"
)

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

func LoadGopherlinkConfig() structs.GopherlinkConfig {
	return structs.GopherlinkConfig{
		Host:     os.Getenv("GOPHERLINKURI"),
		Password: os.Getenv("GOPHERLINKPASS"),
	}
}

func AddGuild(g *discordgo.Guild, invokedByUserID string) *structs.Config {
	config := MakeConfig(g, invokedByUserID)

	AddConfig(&MongoGuild{
		GuildID: g.ID,
		Config:  config,
	})
	return config
}

func ExportConfigYAML(guildid string) ([]byte, error) {
	config, err := GetConfig(guildid)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return nil, err
	}

	if config == nil {
		err = errors.New("config is nil")
	}

	return yaml.Marshal(config)
}

func ImportConfigYAML(guildid string, in []byte) error {
	config := &structs.Config{}
	err := yaml.Unmarshal(in, config)
	if err != nil {
		return err
	}

	_, err = SetConfig(guildid, config)
	return err
}

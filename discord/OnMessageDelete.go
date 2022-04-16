package discord

import (
	"fmt"
	"log"

	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/logging"
	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

var r *redis.Client

func (bot *Bot) OnMessageDelete(s *discordgo.Session, md *discordgo.MessageDelete) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("exemptmessages:%v", md.GuildID)
	request := r.HExists(r.Context(), key, md.ID)
	result, err := request.Result()
	if err != nil {
		log.Println(err)
		result = false // assume its not there if error
	} else {
		request := r.HDel(r.Context(), key, md.ID)
		_, err := request.Result()
		if err != nil {
			log.Println(err)
		}
	}

	conf, err := db.GetConfig(md.GuildID)
	if err != nil || conf == nil {
		return
	}

	if md.BeforeDelete != nil && !result {
		if md.ChannelID == conf.Modules.Logging.ChannelID {
			return
		}
		logging.LogMessageDelete(s, md.BeforeDelete)
	} // not cached otherwise
}

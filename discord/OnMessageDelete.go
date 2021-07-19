package discord

import (
	"fmt"
	"log"

	"github.com/blackmesadev/black-mesa/logging"
	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

var r *redis.Client

func (bot *Bot) OnMessageDelete(s *discordgo.Session, m *discordgo.MessageDelete) {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("exemptmessages:%v", m.GuildID)
	request := r.HExists(r.Context(), key, m.ID)
	result, err := request.Result()
	if err != nil {
		log.Println(err)
		result = false // assume its not there if error
	} else {
		request := r.HDel(r.Context(), key, m.ID)
		_, err := request.Result()
		if err != nil {
			log.Println(err)
		}
	}

	if m.BeforeDelete != nil && !result {
		logging.LogMessageDelete(s, m.BeforeDelete)
	} // not cached otherwise
}

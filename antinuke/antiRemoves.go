package antinuke

import (
	"fmt"
	"time"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

var r *redis.Client

func AntiRemoveProcess(audit *discordgo.AuditLogEntry, guildID string, max int64, timeLimit time.Duration) bool {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("anti:maxRemoves:%v:%v", guildID, audit.UserID)

	res, err := r.Get(r.Context(), key).Int64()
	if err != nil {
		if err == redis.Nil {
			r.Set(r.Context(), key, 1, timeLimit)
			res = 1
		} else {
			fmt.Println(err)
			return true
		}
	}

	r.Incr(r.Context(), key)

	if res > max {
		return false
	}

	return true
}

package antinuke

import (
	"fmt"
	"time"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

var r *redis.Client

func AntiRemoveProcess(s *discordgo.Session, anti structs.AntiNukeThreshold, userID string, guildID string) bool {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("anti:maxRemoves:%v:%v", guildID, userID)

	res, err := r.Get(r.Context(), key).Int64()
	if err != nil {
		if err == redis.Nil {
			r.Set(r.Context(), key, 1, time.Duration(anti.Interval)*time.Second)
			res = 1
		} else {
			fmt.Println(err)
			return true
		}
	}

	r.Incr(r.Context(), key)

	if res > anti.Max {
		return false

	}

	return true
}

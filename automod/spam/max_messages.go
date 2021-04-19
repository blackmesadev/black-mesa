package spam

import (
	"fmt"
	"time"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/go-redis/redis/v8"
)


var r *redis.Client

func ProcessMaxMessages(userId string, guildId string, max int, timeLimit time.Duration, resetOnContinuedSpam bool) bool {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("spam:maxMessages:%v:%v", guildId, userId)

	res, err := r.Get(r.Context(), key).Int()
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
	if resetOnContinuedSpam {
		r.Expire(r.Context(), key, timeLimit) // reset the time limit if they continue spamming
	}

	if res > max {
		return false
	}

	return true
}
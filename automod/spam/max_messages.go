package spam

import (
	"fmt"
	"time"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/go-redis/redis/v8"
)

var r = bmRedis.GetRedis()

func ProcessMaxMessages(userId string, guildId string, max int64, timeLimit time.Duration, resetOnContinuedSpam bool) bool {

	key := fmt.Sprintf("spam:maxMessages:%v:%v", guildId, userId)

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
	if resetOnContinuedSpam {
		r.Expire(r.Context(), key, timeLimit) // reset the time limit if they continue spamming
	}

	if res > max {
		return false
	}

	return true
}

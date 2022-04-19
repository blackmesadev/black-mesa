package util

import (
	"io/ioutil"
	"strconv"
	"time"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

var r *redis.Client

func CalcStats(s *discordgo.Session) {
	r = bmRedis.GetRedis()

	ticker := time.NewTicker(time.Second * 15)

	for {

		select {
		case <-ticker.C:
			// Sys Stats
			// read from /sys/fs/cgroup/memory/memory.usage_in_bytes

			var usedMem float64
			// open file
			m, err := ioutil.ReadFile("/sys/fs/cgroup/memory/memory.usage_in_bytes")
			if err != nil {
				usedMem = 0
			}

			// convert to int
			memInt, err := strconv.Atoi(string(m))
			if err != nil {
				usedMem = 0
			}
			// convert bytes to MB
			if usedMem > 0 {
				usedMem = float64(memInt) / 1024 / 1024
			}

			// Discord stats

			var memberCount int

			for _, i := range s.State.Guilds {
				memberCount += i.MemberCount
			}

			// Send everything to Redis

			r.Set(r.Context(), "usedMem", usedMem, time.Minute)

			r.Set(r.Context(), "memberCount", memberCount, time.Minute)
		}
	}

}

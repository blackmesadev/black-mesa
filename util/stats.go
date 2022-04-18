package util

import (
	"time"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/mem"
)

var r *redis.Client

func CalcStats(s *discordgo.Session) {
	r = bmRedis.GetRedis()

	ticker := time.NewTicker(time.Second * 15)

	for {

		select {
		case <-ticker.C:
			// Sys Stats
			v, _ := mem.VirtualMemory()

			usedMem := v.UsedPercent

			c, _ := cpu.Percent(time.Duration(1*time.Second), true)

			var usedCpu float64

			for _, i := range c {
				usedCpu += i
			}

			usedCpu /= float64(len(c))

			// Discord stats

			var memberCount int

			for _, i := range s.State.Guilds {
				memberCount += i.MemberCount
			}

			// Send everything to Redis

			r.Set(r.Context(), "usedMem", usedMem, time.Minute)

			r.Set(r.Context(), "usedCpu", usedCpu, time.Minute)

			r.Set(r.Context(), "memberCount", memberCount, time.Minute)
		}
	}

}

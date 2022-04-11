package info

import (
	"fmt"
	"runtime"
	"strconv"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

const (
	VERSION = "0.20.1"
)

var r *redis.Client

func BotInfoCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	start := time.Now()

	if r == nil {
		r = bmRedis.GetRedis()
	}

	memResult := r.Get(r.Context(), "usedMem")
	usedMem, err := memResult.Float64()

	if err != nil {
		usedMem = 0
	}

	cpuResult := r.Get(r.Context(), "usedCpu")
	usedCpu, err := cpuResult.Float64()

	if err != nil {
		usedCpu = 0
	}

	membersResult := r.Get(r.Context(), "memberCount")
	membersNum, err := membersResult.Int()

	if err != nil {
		membersNum = 0
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", VERSION, runtime.Version()),
	}

	fields := []*discordgo.MessageEmbedField{
		{
			Name:   "Total Guilds",
			Value:  strconv.Itoa(len(s.State.Guilds)),
			Inline: true,
		},
		{
			Name:   "Total Members",
			Value:  strconv.Itoa(membersNum),
			Inline: true,
		},
		{
			Name:   "Bot Version",
			Value:  VERSION,
			Inline: true,
		},
		{
			Name:   "Go Version",
			Value:  runtime.Version(),
			Inline: true,
		},
		{
			Name:   "Memory Usage",
			Value:  fmt.Sprintf("%.3f", usedMem) + "% Used",
			Inline: true,
		},
		{
			Name:   "CPU Usage",
			Value:  fmt.Sprintf("%.3f", usedCpu) + "% Used",
			Inline: true,
		},
	}
	embed := &discordgo.MessageEmbed{
		URL:    consts.WEBSITE,
		Type:   discordgo.EmbedTypeRich,
		Title:  "Black Mesa Info",
		Color:  0,
		Footer: footer,
		Fields: fields,
	}

	s.ChannelMessageSendEmbed(m.ChannelID, embed)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

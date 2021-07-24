package info

import (
	"fmt"
	"runtime"
	"strconv"

	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

const VERSION = "0.5.2"

const WEBSITE = "https://blackmesa.bot"

var r *redis.Client

func BotInfoCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {

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

	membersResult := r.Get(r.Context(), "members")
	membersNum, err := membersResult.Int()

	if err != nil {
		usedCpu = 0
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
		URL:    WEBSITE,
		Type:   discordgo.EmbedTypeRich,
		Title:  "Black Mesa Info",
		Color:  0,
		Footer: footer,
		Fields: fields,
	}

	s.ChannelMessageSendEmbed(m.ChannelID, embed)
}

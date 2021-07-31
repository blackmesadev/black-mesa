package misc

import (
	"fmt"
	"time"

	"github.com/blackmesadev/discordgo"
)

func PingCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	recievedTime := time.Now()

	discordLatency := s.HeartbeatLatency().Milliseconds()
	pingMsg := fmt.Sprintf("Ping Statistics: Discord:`%vms` ", discordLatency)

	msgTime, err := m.Timestamp.Parse()
	if err != nil {
		pingMsg += "Bot:`Unknown`"
	} else {
		botLatency := recievedTime.Sub(msgTime)
		pingMsg += fmt.Sprintf("Bot:`%vms`", botLatency.Milliseconds())
	}

	s.ChannelMessageSend(m.ChannelID, pingMsg)
}

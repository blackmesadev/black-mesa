package misc

import (
	"fmt"
	"time"

	"github.com/blackmesadev/discordgo"
)

func PingCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	recievedTime := time.Now()

	discordLatency := s.HeartbeatLatency().Milliseconds()
	botLatency := recievedTime.Sub(m.Timestamp).Milliseconds()
	pingMsg := fmt.Sprintf("Ping Statistics: Discord:`%vms` Bot:`%vms`", discordLatency, botLatency)

	s.ChannelMessageSend(m.ChannelID, pingMsg)
}

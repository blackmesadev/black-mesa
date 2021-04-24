package misc

import (
	"fmt"

	"github.com/blackmesadev/discordgo"
)

func PingCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	discordLatency := s.HeartbeatLatency().Milliseconds()
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Ping Statistics: Discord:`%vms`", discordLatency))
}

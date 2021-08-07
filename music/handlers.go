package music

import (
	"log"

	"github.com/blackmesadev/discordgo"
)

func VoiceUpdate(s *discordgo.Session, vu *discordgo.VoiceServerUpdate) {
	err := conn.UpdateVoice(vu.GuildID, s.State.SessionID, vu.Token, vu.Endpoint)
	if err != nil {
		log.Printf("Failed to update voice server on %v: %v\n", vu.GuildID, err)
	}
}

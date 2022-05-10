package music

import (
	"context"
	"log"

	"github.com/blackmesadev/discordgo"
	gopherlink "github.com/damaredayo/gopherlink/proto"
)

func VoiceUpdate(s *discordgo.Session, v *discordgo.VoiceServerUpdate) {
	vc := &gopherlink.DiscordVoiceServer{
		Token:     v.Token,
		GuildId:   v.GuildID,
		UserId:    s.State.User.ID,
		Endpoint:  v.Endpoint,
		SessionId: s.State.SessionID,
	}

	_, err := g.CreatePlayer(context.Background(), vc)
	if err != nil {
		log.Println(err)
		return
	}
}

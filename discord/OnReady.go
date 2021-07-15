package discord

import (
	"fmt"
	"time"

	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnReady(s *discordgo.Session, r *discordgo.Ready) {
	fmt.Printf("Black Mesa ready at %vUTC\nGuilds: %v\nRunning on account ID %v (%v).",
		time.Now().UTC(), len(r.Guilds), r.User.ID, r.User.String())
}

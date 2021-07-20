package discord

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnReady(s *discordgo.Session, r *discordgo.Ready) {
	if util.IsDevInstance(s) {
		s.Debug = true
	}
	fmt.Printf("Black Mesa ready at %v\nGuilds: %v\nRunning on account ID %v (%v).",
		time.Now().UTC(), len(r.Guilds), r.User.ID, r.User.String())
}

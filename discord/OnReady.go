package discord

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/music"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnReady(s *discordgo.Session, r *discordgo.Ready) {
	if util.IsDevInstance(s) {
		s.Debug = true
	}

	idle := 0
	s.UpdateStatusComplex(discordgo.UpdateStatusData{
		IdleSince: &idle,
		Activities: []*discordgo.Activity{
			{
				Name:      fmt.Sprintf("Black Mesa %v", info.VERSION),
				Type:      discordgo.ActivityTypeCustom,
				URL:       info.WEBSITE,
				CreatedAt: time.Now(),
			},
		},
		AFK:    false,
		Status: fmt.Sprintf("Black Mesa %v", info.VERSION),
	})

	music.LavalinkInit(r, config.LoadLavalinkConfig())

	fmt.Printf("Black Mesa ready at %v\nGuilds: %v\nRunning on account ID %v (%v).",
		time.Now().UTC(), len(r.Guilds), r.User.ID, r.User.String())
}

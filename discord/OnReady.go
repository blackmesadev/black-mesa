package discord

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/modules/automod/censor"
	"github.com/blackmesadev/black-mesa/modules/music"
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
				URL:       consts.WEBSITE,
				CreatedAt: time.Now(),
			},
		},
		AFK:    false,
		Status: fmt.Sprintf("Black Mesa %v", info.VERSION),
	})

	music.LavalinkInit(r, db.LoadLavalinkConfig())

	bot.InitRegex()
	censor.StartFlushRegexCache()

	fmt.Printf("Black Mesa ready at %v\nGuilds: %v\nRunning on account ID %v (%v).",
		time.Now().UTC(), len(r.Guilds), r.User.ID, r.User.String())
}

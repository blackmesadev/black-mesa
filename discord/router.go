package discord

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/moderation"
)

func (r *Mux) InitRouter() {
	// Command Router

	// misc
	r.Route("help", misc.HelpCmd)
	r.Route("invite", misc.InviteCmd)

	// config
	r.Route("setup", config.SetupCmd)
	r.Route("get", config.GetConfigCmd)
	r.Route("set", config.SetConfigCmd)

	// moderation
	r.Route("kick", moderation.KickCmd)
	r.Route("ban", moderation.BanCmd)
}

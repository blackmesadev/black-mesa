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
	r.Route("ping", misc.PingCmd)

	// config
	r.Route("setup", config.SetupCmd)
	r.Route("get", config.GetConfigCmd)
	r.Route("set", config.SetConfigCmd)
	r.Route("makemute", config.MakeMuteCmd)

	// moderation
	r.Route("kick", moderation.KickCmd)
	r.Route("ban", moderation.BanCmd)
	r.Route("softban", moderation.SoftBanCmd)
	r.Route("unban", moderation.UnbanCmd)
	r.Route("mute", moderation.MuteCmd)
	r.Route("unmute", moderation.UnmuteCmd)
}

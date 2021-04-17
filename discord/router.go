package discord

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/misc"
)

func (r *Mux) InitRouter() {
	// Command Router
	r.Route("help", misc.HelpCmd)

	r.Route("setup", config.SetupCmd)

	r.Route("get", config.GetConfigCmd)

	r.Route("set", config.SetConfigCmd)
}

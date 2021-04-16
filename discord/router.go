package discord

import "github.com/blackmesadev/black-mesa/misc"

func (r *Mux) InitRouter() {
	// Command Router
	r.Route("help", misc.HelpCmd)
}

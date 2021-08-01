package discord

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/guilds/roles"
	"github.com/blackmesadev/black-mesa/info"
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
	r.Route("strike", moderation.StrikeCmd)
	r.Route("search", moderation.SearchCmd)
	r.Route("purge", moderation.PurgeCmd)

	// moderation funny commands
	r.Route("fuckoff", moderation.KickCmd)
	r.Route("shutup", moderation.MuteCmd)

	// administration
	r.Route("remove", moderation.RemoveActionCmd)

	// info
	r.Route("botinfo", info.BotInfoCmd)

	r.Route("guildinfo", info.GuildInfoCmd)
	r.Route("serverinfo", info.GuildInfoCmd)

	r.Route("userinfo", info.UserInfoCmd)
	r.Route("memberinfo", info.UserInfoCmd)

	//roles
	r.Route("addrole", roles.AddRoleCmd)
}

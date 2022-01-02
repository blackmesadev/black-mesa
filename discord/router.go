package discord

import (
	"github.com/blackmesadev/black-mesa/admin"
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/guilds"
	"github.com/blackmesadev/black-mesa/guilds/permissions"
	"github.com/blackmesadev/black-mesa/guilds/roles"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/moderation"
	"github.com/blackmesadev/black-mesa/music"
)

func (r *Mux) InitRouter() {
	// Command Router

	// admin
	r.Route("adminleave", admin.LeaveCmd)
	r.Route("forcelevel", admin.ForceLevelCmd)

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
	r.Route("clean", moderation.PurgeCmd)

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

	// guilds
	r.Route("prefix", guilds.PrefixCmd)

	r.Route("level", permissions.GetUserLevelCmd)
	r.Route("getlevel", permissions.GetUserLevelCmd)

	r.Route("cmdlevel", permissions.GetCommandLevelCmd)
	r.Route("getcmdlevel", permissions.GetCommandLevelCmd)

	//roles
	r.Route("addrole", roles.AddRoleCmd)

	// music
	r.Route("play", music.PlayCmd)
	r.Route("stop", music.StopCmd)

	r.Route("dc", music.DisconnectCmd)
	r.Route("disconnect", music.DisconnectCmd)

	r.Route("np", music.NowPlayingCmd)
	r.Route("nowplaying", music.NowPlayingCmd)

	r.Route("seek", music.SeekCmd)
	r.Route("goto", music.SeekCmd)

	r.Route("vol", music.VolumeCmd)
	r.Route("volume", music.VolumeCmd)

	r.Route("forward", music.ForwardCmd)

	r.Route("back", music.BackwardCmd)
	r.Route("backward", music.BackwardCmd)

	r.Route("queue", music.QueueCmd)

	r.Route("skip", music.SkipCmd)
}

package discord

import (
	"github.com/blackmesadev/black-mesa/admin"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/misc"

	"github.com/blackmesadev/black-mesa/modules/guilds"
	"github.com/blackmesadev/black-mesa/modules/guilds/permissions"
	"github.com/blackmesadev/black-mesa/modules/guilds/roles"
	"github.com/blackmesadev/black-mesa/modules/moderation"
	"github.com/blackmesadev/black-mesa/modules/music"
	"github.com/blackmesadev/black-mesa/modules/voting"
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
	r.Route("setup", db.SetupCmd)
	r.Route("makemute", db.MakeMuteCmd)

	// moderation
	r.Route("kick", moderation.KickCmd)
	r.Route("ban", moderation.BanCmd)
	r.Route("softban", moderation.SoftBanCmd)
	r.Route("unban", moderation.UnbanCmd)
	r.Route("mute", moderation.MuteCmd)
	r.Route("unmute", moderation.UnmuteCmd)
	r.Route("strike", moderation.StrikeCmd)
	r.Route("mutewithstrike", moderation.MuteWithStrikeCmd)
	r.Route("mws", moderation.MuteWithStrikeCmd)
	r.Route("search", moderation.SearchCmd)
	r.Route("purge", moderation.PurgeCmd)
	r.Route("cancelpurge", moderation.CancelPurgeCmd)
	r.Route("purgecancel", moderation.CancelPurgeCmd)
	r.Route("clean", moderation.PurgeCmd)

	// trusted moderation
	r.Route("banfile", moderation.BanFileCmd)

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
	r.Route("removerole", roles.RemoveRoleCmd)
	r.Route("rmrole", roles.RemoveRoleCmd)
	r.Route("createrole", roles.CreateRoleCmd)
	r.Route("mkrole", roles.CreateRoleCmd)

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

	r.Route("remove", music.RemoveCmd)

	// voting
	r.Route("votemute", voting.StartVoteMuteCmd)

}

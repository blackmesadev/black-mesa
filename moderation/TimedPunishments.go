package moderation

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/mongodb"
)

func AddTimedBan(guildid string, issuer string, userid string, expiry int64) error {
	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		Type:    "ban",
		Expires: expiry,
	}

	_, err := config.AddAction(punishment)
	return err
}

func AddTimedMute(guildid string, issuer string, userid string, roleid string, expiry int64, reason string) error {
	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		RoleID:  roleid,
		Type:    "mute",
		Expires: expiry,
		Reason:  reason,
	}

	_, err := config.AddAction(punishment)
	return err
}

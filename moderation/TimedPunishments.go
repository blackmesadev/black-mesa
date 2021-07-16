package moderation

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/mongodb"
)

func AddTimedBan(guildid string, issuer string, userid string, expiry int64) error {
	punishment := &mongodb.MongoPunishment{
		GuildID:        guildid,
		UserID:         userid,
		Issuer:         issuer,
		PunishmentType: "ban",
		Expires:        expiry,
	}

	_, err := config.AddPunishment(punishment)
	return err
}

func AddTimedRole(guildid string, issuer string, userid string, roleid string, expiry int64, reason string) error {
	punishment := &mongodb.MongoPunishment{
		GuildID:        guildid,
		UserID:         userid,
		Issuer:         issuer,
		RoleID:         roleid,
		PunishmentType: "role",
		Expires:        expiry,
		Reason:         reason,
	}

	_, err := config.AddPunishment(punishment)
	return err
}

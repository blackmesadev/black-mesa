package moderation

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/mongodb"
)

func AddTimedBan(guildid string, userid string, expiry int64) error {
	punishment := &mongodb.MongoPunishment{
		GuildID:        guildid,
		UserID:         userid,
		PunishmentType: "ban",
		Expires:        expiry,
	}

	_, err := config.AddPunishment(punishment)
	return err
}

func AddTimedRole(guildid string, userid string, roleid string, expiry int64) error {
	punishment := &mongodb.MongoPunishment{
		GuildID:        guildid,
		UserID:         userid,
		RoleID:         roleid,
		PunishmentType: "role",
		Expires:        expiry,
	}

	_, err := config.AddPunishment(punishment)
	return err
}
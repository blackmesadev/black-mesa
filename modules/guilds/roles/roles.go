package roles

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/mongodb"
)

func AddTimedRole(guildid string, issuer string, userid string, roleid string, expiry int64) error {
	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		RoleID:  roleid,
		Type:    "role",
		Expires: expiry,
	}

	_, err := config.AddAction(punishment)
	return err
}

func AddRole(guildid string, issuer string, userid string, roleid string) error {
	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		RoleID:  roleid,
		Type:    "role",
	}

	_, err := config.AddAction(punishment)
	return err
}

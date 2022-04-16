package roles

import (
	"github.com/blackmesadev/black-mesa/db"
)

func AddTimedRole(guildid string, issuer string, userid string, roleid string, expiry int64) error {
	punishment := &db.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		RoleID:  roleid,
		Type:    "role",
		Expires: expiry,
	}

	_, err := db.AddAction(punishment)
	return err
}

func AddRole(guildid string, issuer string, userid string, roleid string) error {
	punishment := &db.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		RoleID:  roleid,
		Type:    "role",
	}

	_, err := db.AddAction(punishment)
	return err
}

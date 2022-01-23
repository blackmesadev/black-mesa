package moderation

import (
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/mongodb"
	"go.mongodb.org/mongo-driver/mongo"
)

func AddTimedBan(guildid string, issuer string, userid string, expiry int64, reason string, uuid string) error {
	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		Type:    "ban",
		Expires: expiry,
		Reason:  reason,
		UUID:    uuid,
	}

	_, err := config.AddAction(punishment)
	return err
}

func AddTimedMute(guildid string, issuer string, userid string, roleid string, expiry int64, reason string, uuid string, roles *[]string) error {
	currentMute, err := config.GetMute(guildid, userid)
	if err != nil && err != mongo.ErrNoDocuments {
		return err
	}
	config.DeleteMute(guildid, userid) // delete existing mutes

	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		RoleID:  roleid,
		Type:    "mute",
		Expires: expiry,
		Reason:  reason,
		UUID:    uuid,
	}

	if currentMute != nil {
		roles = currentMute.ReturnRoles
	}

	if roles != nil {
		punishment.ReturnRoles = roles
	}

	_, err = config.AddAction(punishment)
	return err
}

func AddKick(guildid string, issuer string, userid string, reason string, uuid string) error {
	punishment := &mongodb.Action{
		GuildID: guildid,
		UserID:  userid,
		Issuer:  issuer,
		Type:    "kick",
		Reason:  reason,
		UUID:    uuid,
	}

	_, err := config.AddAction(punishment)
	return err
}

package moderation

import (
	"fmt"

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

func AddTimedMute(guildid string, issuer string, userid string, roleid string, expiry int64, reason string, uuid string) (MuteResult, error) {
	res := MuteSuccess

	currentMute, err := config.GetMute(guildid, userid)
	if err != nil && err != mongo.ErrNoDocuments {
		return MuteFailed, err
	}

	if currentMute != nil {
		res = MuteAlreadyMuted
		if issuer == "AutoMod" {
			return res, fmt.Errorf("user already muted during automod")
		}
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

	_, err = config.AddAction(punishment)

	if err != nil {
		return MuteFailed, err
	}

	return res, err
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

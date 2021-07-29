package moderation

import (
	"context"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

func parseCommand(cmd string) ([]string, int64, string) {
	var reason string

	idList := util.SnowflakeRegex.FindAllString(cmd, -1)

	params := util.SnowflakeRegex.Split(cmd, -1)

	if params[len(params)-1][:1] == ">" {
		reason = params[len(params)-1][1:]
	} else {
		reason = params[len(params)-1]
	}

	durationStr := strings.Fields(reason)[0]
	duration := util.ParseTime(durationStr)

	reason = strings.ReplaceAll(reason, durationStr, "")

	reason = strings.TrimSpace(reason)

	fmt.Println(idList)
	fmt.Println(duration)
	fmt.Println(params)
	fmt.Println(reason)

	return idList, duration, reason
}

func IssueStrike(s *discordgo.Session, guildId string, userId string, issuer string, weight int64, reason string, expiry int64, location string, infractionUUID string) error {
	strike := &mongodb.Action{
		GuildID: guildId,
		UserID:  userId,
		Issuer:  issuer,
		Weight:  weight,
		Reason:  reason,
		Expires: expiry,
		Type:    "strike",
		UUID:    infractionUUID,
	}

	_, err := config.AddAction(strike)

	if err != nil {
		return err
	}

	var user *discordgo.User

	member, err := s.State.Member(guildId, userId)
	if err != nil {
		user, err = s.User(userId)
		if err != nil {
			return err
		}
	} else {
		user = member.User
	}

	logging.LogStrike(s, guildId, issuer, user, weight, reason, location, infractionUUID)

	// escalate punishments
	guildConfig, err := config.GetConfig(guildId)
	if err != nil {
		return err
	}

	db := config.GetDB().GetMongoClient().Database("black-mesa").Collection("actions")

	strikeDocs, err := db.Find(context.TODO(), bson.M{
		"guildID": guildId,
		"userID":  userId,
		"type":    "strike",
	})

	if err != nil {
		return err
	}

	strikeTotalWeight := int64(0)

	for strikeDocs.Next(context.TODO()) {
		doc := mongodb.Action{}
		strikeDocs.Decode(&doc)
		strikeTotalWeight += doc.Weight
	}

	strikeEscalationConfig := guildConfig.Modules.Moderation.StrikeEscalation

	i := 0
	strikeEscalationLevels := make([]int64, len(strikeEscalationConfig))
	for k := range strikeEscalationConfig {
		strikeEscalationLevels[i] = k
		i++
	}

	if escalatingTo, ok := strikeEscalationConfig[util.GetClosestLevel(strikeEscalationLevels, strikeTotalWeight)]; ok {
		duration := util.ParseTime(escalatingTo.Duration)

		member, err := s.State.Member(guildId, userId)
		if err == discordgo.ErrStateNotFound {
			member, err = s.GuildMember(guildId, userId)
			if err != nil {
				logging.LogStrikeEscalationFail(s, guildId, userId, err)
				return err
			}
		}
		if err != nil {
			return err
		}

		switch escalatingTo.Type {
		case "mute":
			err := s.GuildMemberRoleAdd(guildId, userId, guildConfig.Modules.Moderation.MuteRole)
			if err != nil {
				return err
			}
			err = AddTimedMute(guildId, "AutoMod", userId, guildConfig.Modules.Moderation.MuteRole, duration, "Exceeded maximum strikes.", infractionUUID)
			if err != nil {
				return err
			}

			if duration != 0 {
				logging.LogTempMute(s, guildId, "AutoMod", member.User, time.Until(time.Unix(duration, 0)), reason, location)
			} else {
				logging.LogMute(s, guildId, "AutoMod", member.User, reason, location)
			}
			return err
		case "ban":
			err := s.GuildBanCreateWithReason(guildId, userId, reason, 0)
			if err != nil {
				return err
			}
			err = AddTimedBan(guildId, "AutoMod", userId, duration, infractionUUID)
			if err != nil {
				return err
			}

			if duration != 0 {
				logging.LogTempBan(s, guildId, "AutoMod", member.User, time.Until(time.Unix(duration, 0)), reason, location)
			} else {
				logging.LogBan(s, guildId, "AutoMod", member.User, reason, location)
			}
		default:
			log.Printf("%v has unknown punishment escalation type %v\n", guildId, escalatingTo.Type)
		}
	}

	return nil
}

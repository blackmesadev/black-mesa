package moderation

import (
	"context"
	"fmt"
	"log"
	"regexp"
	"strconv"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
)

var userIdRegex = regexp.MustCompile(`^(?:<@!?)?(\d+)>?$`)

var snowflakeRegex = regexp.MustCompile(`([0-9]{17,18})`)

var numberRegex = regexp.MustCompile(`[0-9]*[.]?[0-9]+`)

// seconds to regex for the string of it, makes iteration easier as you can use k as the multiplier for v
var timeRegex = map[int64]*regexp.Regexp{
	1:        regexp.MustCompile(`(\d+)s`),
	60:       regexp.MustCompile(`(\d+)m`),
	3600:     regexp.MustCompile(`(\d+)h`),
	86400:    regexp.MustCompile(`(\d+)d`),
	604800:   regexp.MustCompile(`(\d+)w`),
	2628000:  regexp.MustCompile(`(\d+)mo`),
	31536000: regexp.MustCompile(`(\d+)y`),
}

func parseCommand(cmd string) ([]string, int64, string) {
	var reason string

	idList := snowflakeRegex.FindAllString(cmd, -1)

	params := snowflakeRegex.Split(cmd, -1)

	if params[len(params)-1][:1] == ">" {
		reason = params[len(params)-1][1:]
	} else {
		reason = params[len(params)-1]
	}

	durationStr := strings.Fields(reason)[0]
	duration := parseTime(durationStr)

	reason = strings.ReplaceAll(reason, durationStr, "")

	reason = strings.TrimSpace(reason)

	fmt.Println(idList)
	fmt.Println(duration)
	fmt.Println(params)
	fmt.Println(reason)

	return idList, duration, reason
}

// returns a int64 unix timestamp representative of when the punishment can be lifted
func parseTime(strTime string) int64 {
	var unixTime int64

	unixTime = time.Now().Unix()

	for multiplier, regex := range timeRegex {
		timeValStrSlice := regex.FindAllString(strTime, -1)
		if timeValStrSlice != nil {
			timeVal, err := strconv.ParseInt(numberRegex.FindAllString(timeValStrSlice[0], 1)[0], 10, 32) // will be cast to uint32 so needs to be int32 at heart in an int64 body
			if err != nil {
				fmt.Println(err)
				if strings.Contains(err.Error(), "strconv.ParseInt: parsing") {
					return 0
				}
			}
			unixTime += timeVal * multiplier
		}
	}

	// fallback
	if unixTime == time.Now().Unix() {
		return 0
	}

	return unixTime
}

func IssueStrike(s *discordgo.Session, guildId string, userId string, issuer string, weight int64, reason string, expiry int64, location string) error {
	strike := &mongodb.MongoPunishment{
		GuildID:        guildId,
		UserID:         userId,
		Issuer:         issuer,
		Weight:         weight,
		Reason:         reason,
		Expires:        expiry,
		PunishmentType: "strike",
	}

	_, err := config.AddPunishment(strike)

	if err != nil {
		return err
	}

	member, err := s.State.Member(guildId, userId)
	if err != nil {
		return err
	} // ???

	logging.LogStrike(s, guildId, issuer, member.User, weight, reason, location)

	// escalate punishments
	guildConfig, err := config.GetConfig(guildId)
	if err != nil {
		return err
	} // ???????

	db := config.GetDB().GetMongoClient().Database("black-mesa").Collection("punishments")

	strikeDocs, err := db.Find(context.TODO(), bson.M{
		"guildID":        guildId,
		"userID":         userId,
		"punishmentType": "strike",
	})

	if err != nil {
		return err
	}

	strikeTotalWeight := int64(0)

	for strikeDocs.Next(context.TODO()) {
		doc := mongodb.MongoPunishment{}
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
		duration := parseTime(escalatingTo.Duration)

		user, err := s.State.Member(guildId, userId)
		if err != nil {
			return err
		}

		switch escalatingTo.Type {
		case "mute":
			err := s.GuildMemberRoleAdd(guildId, userId, guildConfig.Modules.Moderation.MuteRole)
			if err != nil {
				return err
			}
			err = AddTimedRole(guildId, "AutoMod", userId, guildConfig.Modules.Moderation.MuteRole, duration)
			if err != nil {
				return err
			}

			if duration != 0 {
				logging.LogTempMute(s, guildId, "AutoMod", user.User, time.Until(time.Unix(duration, 0)), reason, location)
			} else {
				logging.LogMute(s, guildId, "AutoMod", user.User, reason, location)
			}
			return err
		case "ban":
			err := s.GuildBanCreateWithReason(guildId, userId, reason, 0)
			if err != nil {
				return err
			}
			err = AddTimedBan(guildId, "AutoMod", userId, duration)
			if err != nil {
				return err
			}

			if duration != 0 {
				logging.LogTempBan(s, guildId, "AutoMod", user.User, time.Until(time.Unix(duration, 0)), reason, location)
			} else {
				logging.LogBan(s, guildId, "AutoMod", user.User, reason, location)
			}
		default:
			log.Printf("%v has unknown punishment escalation type %v\n", guildId, escalatingTo.Type)
		}
	}

	return nil
}

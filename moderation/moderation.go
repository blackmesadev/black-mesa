package moderation

import (
	"context"
	"fmt"
	"log"
	"runtime"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/mongodb"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
	"go.mongodb.org/mongo-driver/bson"
)

type MuteResult int

const (
	MuteSuccess MuteResult = iota
	MuteFailed
	MuteAlreadyMuted
	MuteAlreadyUnmuted
)

func CreatePunishmentEmbed(member *discordgo.Member, guild *discordgo.Guild, actioner *discordgo.User, reason string, expires *time.Time, permenant bool, punishmentType string) *discordgo.MessageEmbed {
	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	fields := []*discordgo.MessageEmbedField{
		{
			Name:   "Server Name",
			Value:  guild.Name,
			Inline: true,
		},
		{
			Name:  "Actioned by",
			Value: actioner.String(),
		},
		{
			Name:  "Reason",
			Value: reason,
		},
	}

	var expiresString string

	if permenant {
		expiresString = "Forever"
	} else if expires != nil {
		expiresString = expires.Format(time.RFC822)
		fields = append(fields, &discordgo.MessageEmbedField{
			Name:  "Expires",
			Value: expiresString,
		},
		)
	}

	embed := &discordgo.MessageEmbed{
		URL:    consts.WEBSITE,
		Type:   discordgo.EmbedTypeRich,
		Title:  fmt.Sprintf("You have been %v.", punishmentType),
		Color:  0,
		Footer: footer,
		Fields: fields,
	}
	return embed
}

func IssueStrike(s *discordgo.Session, guildId string, userId string, issuer string, weight int64, reason string, expiry int64, location string) error {
	infractionUUID := uuid.New().String()
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
	var issuerUser *discordgo.User

	member, err := s.State.Member(guildId, userId)
	if err != nil {
		user, err = s.User(userId)
		if err != nil {
			return err
		}
	} else {
		user = member.User
	}

	guild, err := s.State.Guild(guildId)
	if err != nil {
		guild, err = s.Guild(guildId)
		if err != nil {
			return err
		}
	}

	issuerMember, err := s.State.Member(guildId, issuer)
	if err != nil || issuerMember == nil {
		issuerUser, err = s.User(issuer)
		if err != nil || issuerUser == nil {
			return err
		}
	} else {
		issuerUser = issuerMember.User
	}

	issuerFull := issuerUser.Username + "#" + issuerUser.Discriminator
	logging.LogStrike(s, guildId, issuerFull, user, weight, reason, location, infractionUUID)

	if err == nil {
		s.UserMessageSendEmbed(userId, CreatePunishmentEmbed(member, guild, issuerUser, reason, nil, false, "Striked"))
	}

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

			res, err := AddTimedMute(guildId, "AutoMod", userId, guildConfig.Modules.Moderation.MuteRole, duration, "Exceeded maximum strikes.", uuid.New().String())
			if res == MuteAlreadyMuted {
				logging.LogError(s, guildId, "AutoMod", "user already muted during automod escalation", err)
			}

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
			err = AddTimedBan(guildId, "AutoMod", userId, duration, reason, uuid.New().String())
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

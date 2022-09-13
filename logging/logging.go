package logging

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/mongo"
)

func addLog(s *discordgo.Session, guildId string, emoji string, line string, public bool, channelId string) {
	cfg, err := db.GetConfig(guildId)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			fmt.Printf("couldn't add log for %v: %v\n", guildId, err)
		}
		return
	}

	if cfg.Modules.Logging.ChannelID == "" { // no channel set up
		return
	}

	s.ChannelMessageSend(cfg.Modules.Logging.ChannelID, fmt.Sprintf("%v %v", emoji, line))

	// leave disabled for now, we can come back to public humiliation mode another time -L
	//if public && cfg.Modules.Automod.PublicHumilation {
	//	go s.ChannelMessageSend(channelId, fmt.Sprintf("%v %v", emoji, line))
	//}
}

func LogMessageCensor(s *discordgo.Session, message *discordgo.Message, reason string) {
	var fullName string
	if message.Author.Username == "" || message.Author.Discriminator == "" {
		u, err := s.User(message.Author.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = message.Author.Username + "#" + message.Author.Discriminator
	}

	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	addLog(s,
		message.GuildID,
		consts.EMOJI_CENSORED_MESSAGE,
		fmt.Sprintf("AutoMod censored message by %v (`%v`) in #%v (`%v`): %v\n```\n%v\n```", fullName, message.Author.ID, channel.Name, channel.ID, reason, escapeBackticks(message.Content)),
		false,
		"",
	)
}

func LogMessageViolation(s *discordgo.Session, message *discordgo.Message, reason string) {
	var fullName string
	if message.Author.Username == "" || message.Author.Discriminator == "" {
		u, err := s.User(message.Author.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = message.Author.Username + "#" + message.Author.Discriminator
	}

	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	addLog(s,
		message.GuildID,
		consts.EMOJI_MESSAGE_VIOLATION,
		fmt.Sprintf("AutoMod deleted message by %v (`%v`) in #%v (`%v`) due to violation %v\n```\n%v\n```", fullName, message.Author.ID, channel.Name, channel.ID, reason, escapeBackticks(message.Content)),
		false,
		"",
	)
}

func LogStrike(s *discordgo.Session, guildId string, actor string, target *discordgo.User, weight int64, reason string, location string, uuid string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_STRIKE,
		fmt.Sprintf("%v issued a strike of UUID `%v` (with weight %v) to %v (`%v`): %v", actor, uuid, weight, fullName, target.ID, reason),
		false,
		"",
	)
}

func LogRemoveAction(s *discordgo.Session, guildId string, actor string, uuid string) {
	addLog(s,
		guildId,
		consts.EMOJI_UNSTRIKE,
		fmt.Sprintf("%v removed an action of UUID `%v`", actor, uuid),
		false,
		"",
	)
}

func LogRoleAdd(s *discordgo.Session, guildId string, actor string, role string, target *discordgo.User, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_MUTE,
		fmt.Sprintf("%v added role %v to %v (`%v`)", actor, role, fullName, target.ID),
		true,
		location,
	)
}

func LogTempRoleAdd(s *discordgo.Session, guildId string, actor string, role string, target *discordgo.User, duration time.Duration, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_MUTE,
		fmt.Sprintf("%v added role %v to %v (`%v`) until <t:%v:f>", actor, role, fullName, target.ID, time.Now().Add(duration).Unix()),
		true,
		location,
	)
}

func LogMute(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_MUTE,
		fmt.Sprintf("%v muted %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogTempMute(s *discordgo.Session, guildId string, actor string, target *discordgo.User, duration time.Duration, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_MUTE,
		fmt.Sprintf("%v muted %v (`%v`) until <t:%v:f>: %v", actor, fullName, target.ID, time.Now().Add(duration).Unix(), reason),
		actor == "AutoMod",
		location,
	)
}

func LogUnmute(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_UNMUTE,
		fmt.Sprintf("%v unmuted %v (`%v`): %v", actor, fullName, target.ID, reason),
		false,
		"",
	)
}

func LogMws(s *discordgo.Session, guildId string, actor string, target *discordgo.User, muteDuration time.Duration, strikeDuration time.Duration, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	msg := fmt.Sprintf("%v muted %v (`%v`)", actor, fullName, target.ID)

	if muteDuration != 0 {
		msg = msg + fmt.Sprintf("until <t:%v:f>", time.Now().Add(muteDuration).Unix())
	}

	msg += " with strike "

	if strikeDuration != 0 {
		msg = msg + fmt.Sprintf("until <t:%v:f>", time.Now().Add(strikeDuration).Unix())
	}

	msg += ": " + reason

	addLog(s,
		guildId,
		consts.EMOJI_MUTE,
		msg,
		actor == "AutoMod",
		location,
	)
}

func LogBan(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_BAN,
		fmt.Sprintf("%v banned %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogTempBan(s *discordgo.Session, guildId string, actor string, target *discordgo.User, duration time.Duration, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_BAN,
		fmt.Sprintf("%v banned %v (`%v`) until <t:%v:f>: %v", actor, fullName, target.ID, time.Now().Add(duration).Unix(), reason),
		actor == "AutoMod",
		location,
	)
}

func LogHackBan(s *discordgo.Session, guildId string, actor string, id string, reason string, location string) {
	addLog(s,
		guildId,
		consts.EMOJI_BAN,
		fmt.Sprintf("%v banned `%v`: %v", actor, id, reason),
		actor == "AutoMod",
		location,
	)
}

func LogHackTempBan(s *discordgo.Session, guildId string, actor string, id string, duration time.Duration, reason string, location string) {
	addLog(s,
		guildId,
		consts.EMOJI_BAN,
		fmt.Sprintf("%v banned `%v` until <t:%v:f>: %v", actor, id, time.Now().Add(duration).Unix(), reason),
		actor == "AutoMod",
		location,
	)
}

func LogUnban(s *discordgo.Session, guildId string, actor string, target string, reason string) {
	addLog(s,
		guildId,
		consts.EMOJI_UNBAN,
		fmt.Sprintf("%v unbanned %v: %v", actor, target, reason),
		false,
		"",
	)
}

func LogSoftBan(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_BAN,
		fmt.Sprintf("%v soft banned %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogHackSoftBan(s *discordgo.Session, guildId string, actor string, id string, reason string, location string) {
	addLog(s,
		guildId,
		consts.EMOJI_BAN,
		fmt.Sprintf("%v soft banned `%v`: %v", actor, id, reason),
		actor == "AutoMod",
		location,
	)
}

func LogKick(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	var fullName string
	if target.Username == "" || target.Discriminator == "" {
		u, err := s.User(target.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = target.Username + "#" + target.Discriminator
	}

	addLog(s,
		guildId,
		consts.EMOJI_KICK,
		fmt.Sprintf("%v kicked %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogMessageDelete(s *discordgo.Session, message *discordgo.Message) {

	var fullName string

	if message.Author.Username == "" || message.Author.Discriminator == "" {
		u, err := s.User(message.Author.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = message.Author.Username + "#" + message.Author.Discriminator
	}

	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	attachments := ""
	for _, v := range message.Attachments {
		attachments += v.URL + " "
	}

	log, err := s.GuildAuditLog(message.GuildID, "", "", 72, 1)
	if err != nil {
		return
	}

	var msgDeleteMemberID string
	if len(log.AuditLogEntries) > 0 && log.AuditLogEntries[0].TargetID == message.Author.ID {
		msgDeleteMemberID = log.AuditLogEntries[0].UserID
	} else {
		msgDeleteMemberID = message.Author.ID
	}

	msgDeleteMember, err := s.GuildMember(message.GuildID, msgDeleteMemberID)
	if err != nil {
		return
	}

	addLog(s,
		message.GuildID,
		consts.EMOJI_MESSAGE_DELETE,
		fmt.Sprintf("Message by %v (`%v`) was deleted from #%v (`%v`) by %v (`%v`)\n```\n%v\n```",
			fullName,
			message.Author.ID,
			channel.Name,
			channel.ID,
			fmt.Sprintf("%v#%v", msgDeleteMember.User.Username, msgDeleteMember.User.Discriminator),
			msgDeleteMemberID,
			escapeBackticks(message.Content)+"\n\n"+attachments),
		false,
		"",
	)
}

func LogMessageUpdate(s *discordgo.Session, message *discordgo.Message, before string) {
	var fullName string
	if message.Author.Username == "" || message.Author.Discriminator == "" {
		u, err := s.User(message.Author.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = message.Author.Username + "#" + message.Author.Discriminator
	}

	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	addLog(s,
		message.GuildID,
		consts.EMOJI_MESSAGE_EDIT,
		fmt.Sprintf("Message by %v (`%v`) in #%v (`%v`) was updated\n**Before**\n`%v`\n**After**\n`%v`", fullName, message.Author.ID, channel.Name, channel.ID, before, escapeBackticks(message.Content)),
		false,
		"",
	)
}

func LogStrikeEscalationFail(s *discordgo.Session, guildid string, targetID string, err error) {
	addLog(s,
		guildid,
		consts.EMOJI_CROSS,
		fmt.Sprintf("Failed to escalate strike for `%v` because `%v`", targetID, err.Error()),
		false,
		"",
	)
}

func LogPurge(s *discordgo.Session, message *discordgo.Message, uuid string) {
	var fullName string
	if message.Author.Username == "" || message.Author.Discriminator == "" {
		u, err := s.User(message.Author.ID)
		if err != nil {
			fullName = "Unknown"
		} else {
			fullName = u.Username + "#" + u.Discriminator
		}
	} else {
		fullName = message.Author.Username + "#" + message.Author.Discriminator
	}

	addLog(s,
		message.GuildID,
		consts.EMOJI_MESSAGE_DELETE,
		fmt.Sprintf("A Purge has occured by `%v` which can be viewed here: %v", fullName, consts.ExternalPurgesEndpoint+uuid),
		false,
		"",
	)
}

func LogError(s *discordgo.Session, guildid string, targetID string, action string, err error) {
	addLog(s,
		guildid,
		consts.EMOJI_CROSS,
		fmt.Sprintf("An unknown error occured: `%v` while targetting `%v` for `%v`", err.Error(), targetID, action),
		false,
		"",
	)
}

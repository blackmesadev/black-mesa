package logging

import (
	"fmt"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/discordgo"
)

func addLog(s *discordgo.Session, guildId string, emoji string, line string, public bool, channelId string) {
	cfg, err := config.GetConfig(guildId)
	if err != nil {
		fmt.Printf("couldn't add log for %v: %v\n", guildId, err)
		return
	}

	if cfg.Modules.Logging.ChannelID == "" { // no channel set up
		return
	}

	s.ChannelMessageSend(cfg.Modules.Logging.ChannelID, fmt.Sprintf("%v %v", emoji, line))

	if public && cfg.Modules.Automod.PublicHumilation {
		s.ChannelMessageSend(channelId, fmt.Sprintf("%v %v", emoji, line))
	}
}

func LogMessageCensor(s *discordgo.Session, message *discordgo.Message, reason string) {
	fullName := message.Author.Username + "#" + message.Author.Discriminator
	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	addLog(s,
		message.GuildID,
		"<:mesaCensoredMessage:832350526695407656>",
		fmt.Sprintf("AutoMod censored message by %v (`%v`) in #%v (`%v`): %v\n```\n%v\n```", fullName, message.Author.ID, channel.Name, channel.ID, reason, message.Content),
		false,
		"",
	)
}

func LogMessageViolation(s *discordgo.Session, message *discordgo.Message, reason string) {
	fullName := message.Author.Username + "#" + message.Author.Discriminator
	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	addLog(s,
		message.GuildID,
		"<:mesaMessageViolation:835504185403375616>",
		fmt.Sprintf("AutoMod deleted message by %v (`%v`) in #%v (`%v`) due to violation %v\n```\n%v\n```", fullName, message.Author.ID, channel.Name, channel.ID, reason, message.Content),
		false,
		"",
	)
}

func LogStrike(s *discordgo.Session, guildId string, actor string, target *discordgo.User, count int, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaStrike:832350526922293269>",
		fmt.Sprintf("%v issued %v strikes to %v (`%v`): %v", actor, count, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogMute(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaMemberMute:835506799331246130>",
		fmt.Sprintf("%v muted %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogTempMute(s *discordgo.Session, guildId string, actor string, target *discordgo.User, duration time.Duration, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaMemberMute:835506799331246130>",
		fmt.Sprintf("%v muted %v (`%v`) until %v: %v", actor, fullName, target.ID, time.Now().Add(duration).UTC().Format("02/01/2006 15:04:05PM"), reason),
		actor == "AutoMod",
		location,
	)
}

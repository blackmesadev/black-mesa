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

	go s.ChannelMessageSend(cfg.Modules.Logging.ChannelID, fmt.Sprintf("%v %v", emoji, line))

	// leave disabled for now, we can come back to public humiliation mode another time -L
	//if public && cfg.Modules.Automod.PublicHumilation {
	//	go s.ChannelMessageSend(channelId, fmt.Sprintf("%v %v", emoji, line))
	//}
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

func LogStrike(s *discordgo.Session, guildId string, actor string, target *discordgo.User, weight int64, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaStrike:832350526922293269>",
		fmt.Sprintf("%v issued a strike (with weight %v) to %v (`%v`): %v", actor, weight, fullName, target.ID, reason),
		false,
		"",
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

func LogUnmute(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaMemberUnmute:835506799393374218>",
		fmt.Sprintf("%v unmuted %v (`%v`): %v", actor, fullName, target.ID, reason),
		false,
		"",
	)
}

func LogBan(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaBan:832350526690820146>",
		fmt.Sprintf("%v banned %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogTempBan(s *discordgo.Session, guildId string, actor string, target *discordgo.User, duration time.Duration, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaBan:832350526690820146>",
		fmt.Sprintf("%v banned %v (`%v`) until %v: %v", actor, fullName, target.ID, time.Now().Add(duration).UTC().Format("02/01/2006 15:04:05PM"), reason),
		actor == "AutoMod",
		location,
	)
}

func LogUnban(s *discordgo.Session, guildId string, actor string, target string, reason string) {
	addLog(s,
		guildId,
		"<:mesaBan:832350526690820146>",
		fmt.Sprintf("%v unbanned %v: %v", actor, target, reason),
		false,
		"",
	)
}

func LogKick(s *discordgo.Session, guildId string, actor string, target *discordgo.User, reason string, location string) {
	fullName := target.Username + "#" + target.Discriminator

	addLog(s,
		guildId,
		"<:mesaKick:832350526778900571>",
		fmt.Sprintf("%v kicked %v (`%v`): %v", actor, fullName, target.ID, reason),
		actor == "AutoMod",
		location,
	)
}

func LogMessageDelete(s *discordgo.Session, message *discordgo.Message) {
	fullName := message.Author.Username + "#" + message.Author.Discriminator

	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	attachments := ""
	for _, v := range message.Attachments {
		attachments += v.URL + " "
	}

	addLog(s,
		message.GuildID,
		"<:mesaMessageDelete:832350526917312562>",
		fmt.Sprintf("Message by %v (`%v`) was deleted from #%v (`%v`)\n```\n%v\n```", fullName, message.Author.ID, channel.Name, channel.ID, message.Content+"\n\n"+attachments),
		false,
		"",
	)
}

func LogMessageUpdate(s *discordgo.Session, message *discordgo.Message, before string) {
	fullName := message.Author.Username + "#" + message.Author.Discriminator

	channel, err := s.Channel(message.ChannelID)
	if err != nil {
		return
	} // ?

	addLog(s,
		message.GuildID,
		"<:mesaMessageEdit:832350527009849344>",
		fmt.Sprintf("Message by %v (`%v`) in #%v (`%v`) was updated\n**Before**\n`%v`\n**After**\n`%v`", fullName, message.Author.ID, channel.Name, channel.ID, before, message.Content),
		false,
		"",
	)
}

package music

import (
	"fmt"
	"runtime"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func PlayCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must specify a URL.", consts.EMOJI_CROSS))
		return
	}

	ok := joinMemberChannel(s, m.ChannelID, m.GuildID, m.Author.ID)
	if !ok {
		return
	}

	playSong(s, m.ChannelID, m.GuildID, args[0])

}

func StopCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	stopSong(s, m.ChannelID, m.GuildID)

}

func DisconnectCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	silentStop(s, m.GuildID)

	destroyPlayer(s, m.ChannelID, m.GuildID)

	s.ChannelVoiceLeave(m.GuildID)

}

func NowPlayingCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	nowPlaying(s, m.ChannelID, m.GuildID)

}

func SeekCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `seek <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	seek(s, m.ChannelID, m.GuildID, args[0])
}

func ForwardCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	currentDuration := getPosition(m.GuildID)

	parsedDuration, err := time.ParseDuration(args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	newDuration := currentDuration + parsedDuration

	rawSeek(s, m.ChannelID, m.GuildID, newDuration)

}

func BackwardCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `backward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	currentDuration := getPosition(m.GuildID)

	parsedDuration, err := time.ParseDuration(args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `backward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	newDuration := currentDuration - parsedDuration

	rawSeek(s, m.ChannelID, m.GuildID, newDuration)

}

func VolumeCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Volume: `%v`", consts.EMOJI_CHECK, getVolume(s, m.ChannelID, m.GuildID)))
		return
	}

	err := setVolume(s, m.ChannelID, m.GuildID, args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Failed to set Volume `%v`", consts.EMOJI_CROSS, err))
		return
	}
	// Use get volume here as a sort of check to the end user that it completed successfully.
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Set Volume to: `%v`", consts.EMOJI_CHECK, getVolume(s, m.ChannelID, m.GuildID)))

}

func QueueCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	queue, err := getQueue(m.GuildID)
	if err != nil {
		if err == ErrEmptyQueue {
			s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v There is no queue!", consts.EMOJI_CROSS))
			return
		}
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Unable to fetch queue `%v`", consts.EMOJI_CROSS, err))
		return
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	var guildName string
	guild, err := s.Guild(m.GuildID)
	if err != nil {
		guildName = m.GuildID
	} else {
		guildName = guild.Name
	}

	embedFields := make([]*discordgo.MessageEmbedField, 0)

	for i, track := range queue {
		_, duration := getTimeString(&track.Info)
		embedFields = append(embedFields, &discordgo.MessageEmbedField{
			Name:   util.ZeroWidth,
			Value:  fmt.Sprintf("`%v:` %v `(%v)`", i, track.Info.Title, duration),
			Inline: false,
		})
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Queue for %v", guildName),
		Fields: embedFields,
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
	}

	s.ChannelMessageSendEmbed(m.ChannelID, embed)

}

func AddQueueCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
}

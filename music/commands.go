package music

import (
	"fmt"
	"strconv"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/discordgo"
)

func PlayCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
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
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	stopSong(s, m.ChannelID, m.GuildID)

}

func DisconnectCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	silentStop(s, m.GuildID)

	destroyPlayer(s, m.ChannelID, m.GuildID)

	s.ChannelVoiceLeave(m.GuildID)

}

func NowPlayingCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	nowPlaying(s, m.ChannelID, m.GuildID)

}

func SeekCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `seek <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	seek(s, m.ChannelID, m.GuildID, args[0])
}

func GotoCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `goto <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	currentDuration := getPosition(m.GuildID)

	parsedDuration, err := time.ParseDuration(args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `goto <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	newDuration := currentDuration + parsedDuration

	rawSeek(s, m.ChannelID, m.GuildID, newDuration)

}

func VolumeCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Volume: `%v`", consts.EMOJI_CHECK, getVolume(s, m.ChannelID, m.GuildID)))
		return
	}

	err := setVolume(s, m.ChannelID, m.GuildID, args[0])
	if err != nil {
		if err.(*strconv.NumError) != nil {
			s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `volume <volume:int>`", consts.EMOJI_CROSS))
			return
		}
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Failed to set Volume `%v`", consts.EMOJI_CROSS, err))
		return
	}
	// Use get volume here as a sort of check to the end user that it completed successfully.
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Set Volume to: `%v`", consts.EMOJI_CHECK, getVolume(s, m.ChannelID, m.GuildID)))

}

package music

import (
	"context"
	"errors"
	"fmt"
	"log"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	gopherlink "github.com/damaredayo/gopherlink/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

var g gopherlink.GopherlinkClient

func GopherlinkInit(r *discordgo.Ready, config structs.GopherlinkConfig) {
	conn, err := grpc.Dial(config.Host, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect to gopherlink: %v", err)
		return
	}
	g = gopherlink.NewGopherlinkClient(conn)

	log.Println("Gopherlink connected.")
}

func joinMemberChannel(s *discordgo.Session, channelID, guildID, userID string) bool {
	id := findMemberChannel(s, guildID, userID)

	if id == "" {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v You must be in a voice channel", consts.EMOJI_CROSS))
		return false
	}

	err := s.ChannelVoiceJoinManual(guildID, id, false, true)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Unable to join your voice channel: `%v`", consts.EMOJI_CROSS, err))
		return false
	}

	return true
}

func findMemberChannel(s *discordgo.Session, guildID, userID string) string {
	guild, err := s.State.Guild(guildID)
	if err != nil {
		return ""
	}
	for _, state := range guild.VoiceStates {
		if strings.EqualFold(userID, state.UserID) {
			return state.ChannelID
		}
	}
	return ""
}

func playSong(s *discordgo.Session, channelID, guildID, identifier string) {
	sa, err := g.AddSong(context.Background(), &gopherlink.SongRequest{
		URL:     identifier,
		GuildId: guildID,
	})
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to add song `%v`", consts.EMOJI_CROSS, err))
		return
	}

	sendPlayEmbed(s, channelID, sa.Info)
}

func stopSong(s *discordgo.Session, channelID, guildID string) error {
	_, err := g.StopSong(context.Background(), &gopherlink.SongStopRequest{
		GuildId: guildID,
	})
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to stop track `%v`", consts.EMOJI_CROSS, err))
		return err
	} else {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Stopped", consts.EMOJI_CHECK))
	}
	return nil
}

func skipSong(s *discordgo.Session, channelID, guildID string) error {
	sr, err := g.Skip(context.Background(), &gopherlink.SkipRequest{
		GuildId: guildID,
	})
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Unable to skip track `%v`", consts.EMOJI_CROSS, err))
	} else {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Skipped `%v - %v`", consts.EMOJI_CHECK, sr.Song.Author, sr.Song.Title))
	}
	sendPlayEmbed(s, channelID, sr.Song)
	return nil
}

func silentStop(s *discordgo.Session, guildID string) error {
	_, err := g.StopSong(context.Background(), &gopherlink.SongStopRequest{
		GuildId: guildID,
	})
	if err != nil {
		return err
	}
	return nil
}

func destroyPlayer(s *discordgo.Session, channelID, guildID string) error {
	// TODO: implement this
	return nil
}

func getTimeString(track *gopherlink.SongInfo) (timeElapsedString string, timeDurationString string) {
	timeDuration := time.Unix(0, track.Duration*int64(time.Second))
	timeElapsed := time.Unix(0, track.Elapsed*int64(time.Second))

	// We only need to do upto a day because thats the limit anyway.
	if timeDuration.Day() > 0 {
		timeDurationString = timeDuration.Format("01:15:04:05")
	}
	if timeDuration.Hour() > 0 {
		timeDurationString = timeDuration.Format("15:04:05")
	} else {
		timeDurationString = timeDuration.Format("04:05")
	}

	if timeElapsed.Day() > 0 {
		timeElapsedString = timeElapsed.Format("01:15:04:05")
	}
	if timeElapsed.Hour() > 0 {
		timeElapsedString = timeElapsed.Format("15:04:05")
	} else {
		timeElapsedString = timeElapsed.Format("04:05")
	}

	return

}

func nowPlaying(s *discordgo.Session, channelID, guildID string) {
	track, err := g.NowPlaying(context.Background(), &gopherlink.NowPlayingRequest{
		GuildId: guildID,
	})
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to get now playing `%v`", consts.EMOJI_CROSS, err))
		return
	}

	timeElapsedString, timeDurationString := getTimeString(track)

	embedFields := []*discordgo.MessageEmbedField{
		{
			Name:   "Author",
			Value:  track.Author,
			Inline: true,
		},
		{
			Name:   "Title",
			Value:  track.Title,
			Inline: true,
		},
		{
			Name:   "Time Elapsed",
			Value:  fmt.Sprintf("%v/%v", timeElapsedString, timeDurationString),
			Inline: true,
		},
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
	}

	embed := &discordgo.MessageEmbed{
		URL:    track.URL,
		Title:  fmt.Sprintf("Playing %v", track.Title),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	s.ChannelMessageSendEmbed(channelID, embed)
}

func seek(s *discordgo.Session, channelID, guildID, duration string) {
	parsedDuration, err := time.ParseDuration(duration)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to parse duration, the format is: `1h30m45s`", consts.EMOJI_CROSS))
		return
	}
	ss, err := g.Seek(context.Background(), &gopherlink.SeekRequest{
		GuildId:  guildID,
		Duration: int64(parsedDuration.Seconds()),
		Type:     gopherlink.SeekType_TO_DURATION,
	})

	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to seek `%v`", consts.EMOJI_CROSS, err))
		return
	}
	newDuration, _ := getTimeString(ss)
	s.ChannelMessageSend(channelID, fmt.Sprintf("%v Seeked to `%v`", consts.EMOJI_CHECK, newDuration))
}

func rawSeek(s *discordgo.Session, channelID, guildID string, duration time.Duration) {
	// TODO: implement this
}

func getPosition(guildID string) *time.Duration {
	np, err := g.NowPlaying(context.Background(), &gopherlink.NowPlayingRequest{
		GuildId: guildID,
	})
	if err != nil {
		return nil
	}
	pos := time.Duration(np.Elapsed * int64(time.Second))
	return &pos
}

func playerInfo(s *discordgo.Session, channelID, guildID string) error {
	np, err := g.NowPlaying(context.Background(), &gopherlink.NowPlayingRequest{
		GuildId: guildID,
	})
	if err != nil {
		return nil
	}

	var status string

	switch np.Playing {
	case gopherlink.PlayStatus_PLAYING:
		status = "Playing"
	case gopherlink.PlayStatus_PAUSED:
		status = "Paused"
	case gopherlink.PlayStatus_STOPPED:
		status = "Stopped"
	}

	embedFields := []*discordgo.MessageEmbedField{
		{
			Name:   "Status",
			Value:  status,
			Inline: true,
		},
		{
			Name:   "Track",
			Value:  np.Title,
			Inline: true,
		},
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Black Mesa Music Status"),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	s.ChannelMessageSendEmbed(channelID, embed)
	return nil

}

func getVolume(s *discordgo.Session, channelID, guildID string) string {
	// TODO: implement this
	return ""
}

func setVolume(s *discordgo.Session, channelID, guildID, volume string) error {
	volumeInt, err := strconv.Atoi(volume)
	if err != nil {
		return err.(*strconv.NumError).Err
	}

	if volumeInt < 0 || volumeInt > 100 {
		return errors.New("Volume is out of range, must be within [0, 100]")
	}

	g.Volume(context.Background(), &gopherlink.VolumeRequest{
		GuildId: guildID,
		Volume:  float32(volumeInt) / 100,
	})
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to set volume. `%v`", consts.EMOJI_CROSS, err))
		return err
	}
	s.ChannelMessageSend(channelID, fmt.Sprintf("%v Set volume to `%v`", consts.EMOJI_CHECK, volumeInt))
	return nil
}

func isInUserVoiceChannel(s *discordgo.Session, guildID, userID string) (channelID string, ok bool) {
	guild, err := s.Guild(guildID)
	if err != nil {
		return "", false
	}

	// Check if the user is in the same voice channel as the bot
	var userChannel string
	var botChannel string
	for _, vs := range guild.VoiceStates {
		if vs.UserID == userID {
			userChannel = vs.ChannelID
		}
		if vs.UserID == s.State.User.ID {
			botChannel = vs.ChannelID
		}
	}

	return botChannel, userChannel == botChannel
}

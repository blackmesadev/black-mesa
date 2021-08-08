package music

import (
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
	"github.com/blackmesadev/gavalink"
)

var (
	lavalink *gavalink.Lavalink
)

func LavalinkInit(r *discordgo.Ready, config structs.LavalinkConfig) {
	lavalink = gavalink.NewLavalink("1", r.User.ID)

	err := lavalink.AddNodes(gavalink.NodeConfig{
		REST:      fmt.Sprintf("http://%s", config.Host),
		WebSocket: fmt.Sprintf("ws://%s", config.Host),
		Password:  config.Password,
	})

	if err != nil {
		log.Println(err)
		return
	}

	log.Println("Lavalink connected.")

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
	node, err := lavalink.BestNode()
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch lavalink node `%v`", consts.EMOJI_CROSS, err))
		return
	}

	tracks, err := node.LoadTracks(identifier)
	if err != nil || tracks.Type != gavalink.TrackLoaded {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to load track `%v`", consts.EMOJI_CROSS, err))
		return
	}

	track := tracks.Tracks[0]
	err = players[guildID].Play(track.Data)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to play track `%v`", consts.EMOJI_CROSS, err))
		return
	}

	timeDuration := time.Millisecond * time.Duration(track.Info.Length)

	embedFields := []*discordgo.MessageEmbedField{
		{
			Name:   "Author",
			Value:  track.Info.Author,
			Inline: true,
		},
		{
			Name:   "Title",
			Value:  track.Info.Title,
			Inline: true,
		},
		{
			Name:   "ID",
			Value:  track.Info.Identifier,
			Inline: true,
		},
		{
			Name:   "Duration",
			Value:  timeDuration.String(),
			Inline: true,
		},
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Playing %v", track.Info.Title),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	s.ChannelMessageSendEmbed(channelID, embed)

}

func stopSong(s *discordgo.Session, channelID, guildID string) error {
	err := players[guildID].Stop()
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to stop track `%v`", consts.EMOJI_CROSS, err))
		return err
	} else {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Stopped", consts.EMOJI_CHECK))
	}
	return nil
}

func silentStop(s *discordgo.Session, guildID string) error {
	err := players[guildID].Stop()
	if err != nil {
		return err
	}
	return nil
}

func destroyPlayer(s *discordgo.Session, channelID, guildID string) error {
	err := players[guildID].Destroy()
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to destroy player `%v`", consts.EMOJI_CROSS, err))
		return err
	}
	return nil
}

func nowPlaying(s *discordgo.Session, channelID, guildID string) {
	track := players[guildID].Track()
	if track == "" {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Nothing playing", consts.EMOJI_CROSS))
		return
	}
	s.ChannelMessageSend(channelID, track)
}

func seek(s *discordgo.Session, channelID, guildID, duration string) {
	parsedDuration, err := time.ParseDuration(duration)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to parse duration, the format is: `1h30m45s`", consts.EMOJI_CROSS))
	}
	err = players[guildID].Seek(int(parsedDuration.Milliseconds()))
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to seek `%v`", consts.EMOJI_CROSS, err))
	}
}

func rawSeek(s *discordgo.Session, channelID, guildID string, duration time.Duration) {
	err := players[guildID].Seek(int(duration.Milliseconds()))
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to seek `%v`", consts.EMOJI_CROSS, err))
	}
}

func getPosition(guildID string) time.Duration {
	return time.Duration(players[guildID].Position()) * time.Millisecond
}

func playerInfo(s *discordgo.Session, channelID, guildID string) {
	player := players[guildID]

	var status string

	if player.Track() == "" {
		status = "Stopped"
	} else if player.Paused() {
		status = "Paused"
	} else {
		status = "Playing"
	}

	embedFields := []*discordgo.MessageEmbedField{
		{
			Name:   "Status",
			Value:  status,
			Inline: true,
		},
		{
			Name:   "Track",
			Value:  player.Track(),
			Inline: true,
		},
		{
			Name:   "Volume",
			Value:  strconv.Itoa(player.GetVolume()),
			Inline: true,
		},
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Black Mesa Music Status"),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	s.ChannelMessageSendEmbed(channelID, embed)

}

func getVolume(s *discordgo.Session, channelID, guildID string) string {
	return strconv.Itoa(players[guildID].GetVolume())
}

func setVolume(s *discordgo.Session, channelID, guildID, volume string) error {
	volumeInt, err := strconv.Atoi(volume)
	if err != nil {
		return err
	}

	err = players[guildID].Volume(volumeInt)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to set volume. `%v`", consts.EMOJI_CROSS, err))
		return err
	}
	return nil
}

package music

import (
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

	next, err := getNext(guildID)
	if err == nil && next != nil {
		player, ok := players[guildID]
		if !ok {
			// if its not here, its likely just due to voice update not finishing yet, so we can implement retry logic
			var count int
			for count < 5 {
				time.Sleep(500 * time.Millisecond)
				player, ok = players[guildID]
				if ok {
					break
				}
				count++
			}
			// test that it worked
			if !ok {
				s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
				return
			}
		}
		track := *next
		err = player.Play(track.Data)
		sendPlayEmbed(s, channelID, track)
	}

	tracks, err := node.LoadTracks(identifier)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to load track `%v`", consts.EMOJI_CROSS, err))
		return
	}

	if tracks.Type == gavalink.PlaylistLoaded {
		trackCount := len(tracks.Tracks)
		for i, track := range tracks.Tracks {
			if i == 0 {
				player, ok := players[guildID]
				if !ok {
					// if its not here, its likely just due to voice update not finishing yet, so we can implement retry logic
					var count int
					for count < 5 {
						time.Sleep(500 * time.Millisecond)
						player, ok = players[guildID]
						if ok {
							break
						}
						count++
					}
					// test that it worked
					if !ok {
						s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
						return
					}
				}
				if player.Track() != "" {
					ok, err := addQueue(guildID, track.Data)
					if !ok || err != nil {
						log.Println("Failed to add track to queue", err, track.Info)
						trackCount--
					}
				} else {
					err = player.Play(track.Data)
					if err != nil {
						s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to play track `%v`", consts.EMOJI_CROSS, err))
						return
					}
				}

			} else {
				ok, err := addQueue(guildID, track.Data)
				if !ok || err != nil {
					log.Println("Failed to add track to queue", err, track.Info)
					trackCount--
				}
			}
		}

		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Queued `%v` tracks successfully", consts.EMOJI_CHECK, trackCount))
	}

	if tracks.Type == gavalink.TrackLoaded {
		player, ok := players[guildID]
		if !ok {
			// if its not here, its likely just due to voice update not finishing yet, so we can implement retry logic
			var count int
			for count < 5 {
				time.Sleep(500 * time.Millisecond)
				player, ok = players[guildID]
				if ok {
					break
				}
				count++
			}
			// test that it worked
			if !ok {
				s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
				return
			}
		}
		track := tracks.Tracks[0]
		if player.Track() != "" {
			ok, err := addQueue(guildID, track.Data)
			if !ok || err != nil {
				log.Println("Failed to add track to queue", err, track.Info)
			} else {
				s.ChannelMessageSend(channelID, fmt.Sprintf("%v Queued `%v`", consts.EMOJI_CHECK, track.Info.Title))
			}
			return
		}
		err = player.Play(track.Data)
		sendPlayEmbed(s, channelID, track)
		if err != nil {
			s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to play track `%v`", consts.EMOJI_CROSS, err))
			return
		}
	}

	if tracks.Type == gavalink.LoadFailed {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to play track `LOAD_FAILED`", consts.EMOJI_CROSS))
		return
	}

	if tracks.Type == gavalink.SearchResult {
		player, ok := players[guildID]
		if !ok {
			// if its not here, its likely just due to voice update not finishing yet, so we can implement retry logic
			var count int
			for count < 5 {
				time.Sleep(500 * time.Millisecond)
				player, ok = players[guildID]
				if ok {
					break
				}
				count++
			}
			// test that it worked
			if !ok {
				s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
				return
			}
		}
		if len(tracks.Tracks) == 0 {
			s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to find track", consts.EMOJI_CROSS))
			return
		}
		track := tracks.Tracks[0]
		if player.Track() != "" {
			ok, err := addQueue(guildID, track.Data)
			if !ok || err != nil {
				log.Println("Failed to add track to queue", err, track.Info)
			} else {
				s.ChannelMessageSend(channelID, fmt.Sprintf("%v Queued `%v`", consts.EMOJI_CHECK, track.Info.Title))
			}
			return
		}
		err = player.Play(track.Data)
		sendPlayEmbed(s, channelID, track)
		if err != nil {
			s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to play track `%v`", consts.EMOJI_CROSS, err))
			return
		}
	}

	if tracks.Type == gavalink.NoMatches {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to find track `NO_MATCHES`", consts.EMOJI_CROSS))
		return
	}
}

func stopSong(s *discordgo.Session, channelID, guildID string) error {
	player, ok := players[guildID]
	if !ok {
		return ErrNoPlayer
	}
	err := player.Stop()
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to stop track `%v`", consts.EMOJI_CROSS, err))
		return err
	} else {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Stopped", consts.EMOJI_CHECK))
	}
	return nil
}

func skipSong(s *discordgo.Session, channelID, guildID string) error {
	player, ok := players[guildID]
	if !ok {
		return ErrNoPlayer
	}

	next, err := getNext(player.GuildID())
	if err != nil {
		return err
	}

	err = player.Play(next.Data)
	if err != nil {
		return err
	}

	sendPlayEmbed(s, channelID, *next)
	return nil
}

func silentStop(s *discordgo.Session, guildID string) error {
	player, ok := players[guildID]
	if !ok {
		return ErrNoPlayer
	}
	err := player.Stop()
	if err != nil {
		return err
	}
	return nil
}

func destroyPlayer(s *discordgo.Session, channelID, guildID string) error {
	player, ok := players[guildID]
	if !ok {
		return ErrNoPlayer
	}
	err := player.Destroy()
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to destroy player `%v`", consts.EMOJI_CROSS, err))
		return err
	}
	return nil
}

func getTimeString(track *gavalink.TrackInfo) (timeElapsedString string, timeDurationString string) {
	timeDuration := time.Unix(0, int64(track.Length*int(time.Millisecond)))
	timeElapsed := time.Unix(0, int64(track.Position*int(time.Millisecond)))

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
	player, ok := players[guildID]
	if !ok {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
		return
	}
	base64Track := player.Track()
	if base64Track == "" {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Nothing playing", consts.EMOJI_CROSS))
		return
	}

	track, err := gavalink.DecodeString(base64Track)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Unable to fetch track info `%v`", consts.EMOJI_CROSS, err))
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
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	embed := &discordgo.MessageEmbed{
		URL:    track.URI,
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
	}
	player, ok := players[guildID]
	if !ok {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
		return
	}
	err = player.Seek(int(parsedDuration.Milliseconds()))
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to seek `%v`", consts.EMOJI_CROSS, err))
	}
}

func rawSeek(s *discordgo.Session, channelID, guildID string, duration time.Duration) {
	player, ok := players[guildID]
	if !ok {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
		return
	}
	err := player.Seek(int(duration.Milliseconds()))
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to seek `%v`", consts.EMOJI_CROSS, err))
	}
}

func getPosition(guildID string) *time.Duration {
	player, ok := players[guildID]
	if !ok {
		return nil
	}
	pos := time.Duration(player.Position()) * time.Millisecond
	return &pos
}

func playerInfo(s *discordgo.Session, channelID, guildID string) {
	player, ok := players[guildID]
	if !ok {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
		return
	}

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
	player, ok := players[guildID]
	if !ok {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
		return ""
	}
	return strconv.Itoa(player.GetVolume())
}

func setVolume(s *discordgo.Session, channelID, guildID, volume string) error {
	player, ok := players[guildID]
	if !ok {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to fetch player, try disconnecting and rejoining the bot to the VC `%v`", consts.EMOJI_CROSS, ErrNoPlayer))
		return ErrNoPlayer
	}

	volumeInt, err := strconv.Atoi(volume)
	if err != nil {
		return err.(*strconv.NumError).Err
	}

	if volumeInt < 0 || volumeInt > 1000 {
		return errors.New("Volume is out of range, must be within [0, 1000]")
	}

	err = player.Volume(volumeInt)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to set volume. `%v`", consts.EMOJI_CROSS, err))
		return err
	}
	return nil
}

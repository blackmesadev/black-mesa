package music

import (
	"context"
	"fmt"
	"log"
	"net/url"
	"runtime"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"github.com/lukasl-dev/waterlink"
	"github.com/lukasl-dev/waterlink/entity/track"
	"github.com/lukasl-dev/waterlink/usecase/play"
)

var (
	conn waterlink.Connection
	req  waterlink.Requester
)

func LavalinkInit(config structs.LavalinkConfig) {
	var err error
	connOpts := waterlink.NewConnectOptions().WithPassphrase(config.Password)
	reqOpts := waterlink.NewRequesterOptions().WithPassphrase(config.Password)

	httpHost, _ := url.Parse(fmt.Sprintf("http://%s", config.Host))
	wsHost, _ := url.Parse(fmt.Sprintf("ws://%s", config.Host))

	conn, err = waterlink.Connect(context.TODO(), *wsHost, connOpts)
	if err != nil {
		log.Println(err)
		return
	}

	req = waterlink.NewRequester(*httpHost, reqOpts)

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
	resp, err := req.LoadTracks(identifier)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to load track `%v`", consts.EMOJI_CROSS, err))
		return
	}

	var loadTrack track.Track
	if len(resp.Tracks) > 0 {
		loadTrack = resp.Tracks[0]
	}

	opts := play.NewOptions().WithVolume(100).WithPaused(false)
	err = conn.Play(guildID, loadTrack.ID, opts)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to play track `%v`", consts.EMOJI_CROSS, err))
		return
	}

	timeDuration := time.Millisecond * time.Duration(loadTrack.Info.Length)

	embedFields := []*discordgo.MessageEmbedField{
		{
			Name:   "Author",
			Value:  loadTrack.Info.Author,
			Inline: true,
		},
		{
			Name:   "Title",
			Value:  loadTrack.Info.Title,
			Inline: true,
		},
		{
			Name:   "ID",
			Value:  loadTrack.Info.Identifier,
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
		Title:  fmt.Sprintf("Playing %v", loadTrack.Info.Title),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	s.ChannelMessageSendEmbed(channelID, embed)

}

func stopSong(s *discordgo.Session, channelID, guildID string) error {
	err := conn.Stop(guildID)
	if err != nil {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Failed to stop track `%v`", consts.EMOJI_CROSS, err))
		return err
	} else {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v Stopped.", consts.EMOJI_CHECK))
	}
	return nil
}

package music

import (
	"fmt"
	"runtime"
	"time"

	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/discordgo"
	"github.com/blackmesadev/gavalink"
)

func sendPlayEmbed(s *discordgo.Session, channelID string, track gavalink.Track) {

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
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
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

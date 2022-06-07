package music

import (
	"fmt"
	"runtime"
	"time"

	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/discordgo"
	gopherlink "github.com/damaredayo/gopherlink/proto"
)

func sendPlayEmbed(s *discordgo.Session, channelID string, track *gopherlink.SongInfo) {

	timeDuration := time.Second * time.Duration(track.Duration)

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
			Name:   "Duration",
			Value:  timeDuration.String(),
			Inline: true,
		},
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Playing %v", track.Title),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	s.ChannelMessageSendEmbed(channelID, embed)
}

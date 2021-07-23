package discord

import (
	"fmt"
	"log"
	"runtime"

	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/discordgo"
)

func ErrorHandler(s *discordgo.Session, channelID string, commandError error) {
	if channelID == "" {
		log.Println(commandError)
	}
	embedFields := make([]*discordgo.MessageEmbedField, 0)
	field := &discordgo.MessageEmbedField{
		Name:   "Error",
		Value:  commandError.Error(),
		Inline: false,
	}
	embedFields = append(embedFields, field)
	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}
	embed := &discordgo.MessageEmbed{
		Title:  "Black Mesa couldn't handle this!",
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}
	_, err := s.ChannelMessageSendEmbed(channelID, embed)
	if err != nil {
		log.Println(err)
		log.Println(commandError)
	}
}

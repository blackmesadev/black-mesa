package discord

import (
	"fmt"
	"log"
	"runtime"

	"github.com/bwmarrin/discordgo"
)

func (bot *Bot) ErrorHandler(s *discordgo.Session, channelID string, commandError error) {
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
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", bot.Version, runtime.Version()),
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

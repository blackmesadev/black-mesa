package main

import (
	"log"
	"strings"

	"github.com/bwmarrin/discordgo"
)

func messageHandler(s *discordgo.Session, m *discordgo.MessageCreate) {
	if m.Author.ID == s.State.User.ID {
		return
	}

	if strings.HasPrefix(m.Content, bot.Prefix) {
		command := strings.Trim(m.Content, bot.Prefix)
		parameters := strings.Fields(command)
		if len(parameters) == 0 { // how the fuck could this even happen tbh
			log.Println("somehow parameters is zero, just whatever who cares ignore this")
			return
		}
		if val, ok := bot.Commands[parameters[0]]; ok {
			val.(func(s *discordgo.Session, m *discordgo.MessageCreate, parameters []string))(s, m, parameters)
		}
	}
}

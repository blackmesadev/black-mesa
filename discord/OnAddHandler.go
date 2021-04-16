package discord

import (
	"log"

	"github.com/bwmarrin/discordgo"
)

func (bot *Bot) onAddHandler(s *discordgo.Session, g *discordgo.GuildCreate) {
	log.Println("Added to", g.ID, g.Name)

}

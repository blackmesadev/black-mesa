package music

import (
	"log"

	"github.com/blackmesadev/discordgo"
	"github.com/blackmesadev/gavalink"
)

var players = make(map[string]*gavalink.Player)

func VoiceUpdate(s *discordgo.Session, vu *discordgo.VoiceServerUpdate) {
	vsu := gavalink.VoiceServerUpdate{
		Endpoint: vu.Endpoint,
		GuildID:  vu.GuildID,
		Token:    vu.Token,
	}

	p, err := lavalink.GetPlayer(vu.GuildID)
	if err != nil {
		log.Println("Unable to get player", vu, err)
		return
	}

	err = p.Forward(s.State.SessionID, vsu)
	if err != nil {
		log.Println("Unable to forward data to player", vu, err)
		return
	}

	node, err := lavalink.BestNode()
	if err != nil {
		log.Println("Unable to fetch best node", err)
		return
	}

	eventHandler := new(gavalink.DummyEventHandler) // dummy for now, will do more with this in the future

	player, err := node.CreatePlayer(vu.GuildID, s.State.SessionID, vsu, eventHandler)

	if err != nil {
		log.Println("Unable to create player", err)
		return
	}

	players[vu.GuildID] = player
}

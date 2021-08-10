package music

import (
	"errors"
	"log"

	"github.com/blackmesadev/discordgo"
	"github.com/blackmesadev/gavalink"
)

var players = make(map[string]*gavalink.Player)

var globalSession *discordgo.Session

var eh interface {
	OnTrackEnd(player *gavalink.Player, track string, reason string) error
	OnTrackException(player *gavalink.Player, track string, reason string) error
	OnTrackStuck(player *gavalink.Player, track string, threshold int) error
}

func VoiceUpdate(s *discordgo.Session, vu *discordgo.VoiceServerUpdate) {
	if globalSession == nil {
		globalSession = s
	}
	vsu := gavalink.VoiceServerUpdate{
		Endpoint: vu.Endpoint,
		GuildID:  vu.GuildID,
		Token:    vu.Token,
	}

	// attempt to get existing player if existing
	if p, err := lavalink.GetPlayer(vu.GuildID); err == nil {
		err = p.Forward(s.State.SessionID, vsu)
		if err != nil {
			log.Println("Unable to forward", vu, err)
		}
		return
	}

	// create a new player then.
	node, err := lavalink.BestNode()
	if err != nil {
		log.Println("Unable to fetch best node", err)
		return
	}

	player, err := node.CreatePlayer(vu.GuildID, s.State.SessionID, vsu, eh)

	if err != nil {
		log.Println("Unable to create player", err)
		return
	}

	players[vu.GuildID] = player
}

func OnTrackEnd(player *gavalink.Player, track string, reason string) error {
	if globalSession == nil {
		return errors.New("no session")
	}

	next, err := getNext(player.GuildID())
	if err != nil {
		return err
	}

	playSong(globalSession, "", player.GuildID(), next.Data)
	return nil
}

func OnTrackException(player *gavalink.Player, track string, reason string) error {
	return nil
}
func OnTrackStuck(player *gavalink.Player, track string, threshold int) error {
	return nil
}

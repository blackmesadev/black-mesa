package music

import (
	"errors"
	"fmt"
	"log"

	"github.com/blackmesadev/discordgo"
	"github.com/blackmesadev/gavalink"
)

var players = make(map[string]*gavalink.Player)

var globalSession *discordgo.Session

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
	eh := gavalink.EventHandler{
		OnTrackEnd:       OnTrackEnd,
		OnTrackException: OnTrackException,
		OnTrackStuck:     OnTrackStuck,
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

	if reason == gavalink.ReasonFinished {
		next, err := getNext(player.GuildID())
		if err != nil {
			return err
		}

		// end of queue
		if next == nil {
			return nil
		}

		err = player.Play(next.Data)
		if err != nil {
			return err
		}
	}
	return nil
}

func OnTrackException(player *gavalink.Player, track string, reason string) error {
	// on a track exception, we just want to try and play the next song in the queue
	if player.Track() != "" {
		removeQueue(player.GuildID(), track)
		return nil
	}
	next, err := getNext(player.GuildID())
	if err != nil {
		return err
	}

	// end of queue
	if next == nil {
		return nil
	}

	err = player.Play(next.Data)
	if err != nil {
		return err
	}

	return nil
}
func OnTrackStuck(player *gavalink.Player, track string, threshold int) error {
	// on a track exception, we just want to try and play the next song in the queue
	fmt.Println("track stuck", track, threshold)
	if player.Track() != "" {
		return nil
	}
	next, err := getNext(player.GuildID())
	if err != nil {
		return err
	}

	// end of queue
	if next == nil {
		return nil
	}

	err = player.Play(next.Data)
	if err != nil {
		return err
	}

	return nil
}

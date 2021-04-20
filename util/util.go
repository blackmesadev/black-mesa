package util

import "github.com/blackmesadev/discordgo"

func IsDevInstance(s *discordgo.Session) bool {
	user, err := s.User("@me")
	if err != nil {
		return false // assume not dev bot
	}

	return user.ID == "832359937526202398"
}
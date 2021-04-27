package util

import "github.com/blackmesadev/discordgo"

func IsDevInstance(s *discordgo.Session) bool {
	user, err := s.User("@me")
	if err != nil {
		return false // assume not dev bot
	}

	return user.ID == "832359937526202398"
}

// Gets the closest level that the ideal level can match in the level -> interface map
func GetClosestLevel(i []int64, targetLevel int64) int64 {
	var closest int64 = 0
	for _, level := range i {
		if level == targetLevel {
			return targetLevel
		}

		if level < targetLevel {
			closest = level
		} else {
			return closest // micro optimization; return early if the level is ever higher than the target
		}
	}

	return closest
}
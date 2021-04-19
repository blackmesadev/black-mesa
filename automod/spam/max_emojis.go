package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxEmojis(message *discordgo.Message, limit int) bool {
	if len(message.GetCustomEmojis()) > limit { // micro optimization; don't even bother with unicode if customs are over
		return false
	}

	unicodeEmojis := 0
	for _, r := range message.Content {
		// various unicode emoji ranges
		if r >= 127744 && r <= 129750 {
			unicodeEmojis++
		} else if r >= 126980 && r <= 127569 {
			unicodeEmojis++
		} else if r >= 169 && r <= 174 {
			unicodeEmojis++
		} else if r >= 8205 && r <= 12953 {
			unicodeEmojis++
		}
	}

	if (len(message.GetCustomEmojis()) + unicodeEmojis) > limit {
		return false
	}

	return true
}
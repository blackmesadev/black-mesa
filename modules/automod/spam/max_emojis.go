package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxEmojis(message *discordgo.Message, limit int64) (bool, int64) {
	if limit == 0 {
		return true, 0
	}
	customEmojis := int64(len(message.GetCustomEmojis()))
	if customEmojis > limit { // micro optimization; don't even bother with unicode if customs are over
		return false, customEmojis
	}

	var unicodeEmojis int64 = 0
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

	total := customEmojis + unicodeEmojis

	if total > limit {
		return false, total
	}

	return true, 0
}

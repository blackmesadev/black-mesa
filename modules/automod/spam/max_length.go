package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxLength(message *discordgo.Message, limit int64) (bool, int64) {
	if limit == 0 {
		return true, 0
	}
	return int64(len(message.Content)) <= limit, int64(len(message.Content))
}

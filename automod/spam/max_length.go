package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxLength(message *discordgo.Message, limit int64) (bool, int64) {
	return int64(len(message.Content)) <= limit, int64(len(message.Content))
}

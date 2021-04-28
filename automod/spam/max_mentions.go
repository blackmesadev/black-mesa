package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxMentions(message *discordgo.Message, limit int64) (bool, int64) {
	return int64(len(message.Mentions)) <= limit, int64(len(message.Mentions))
}

func ProcessMaxRoleMentions(message *discordgo.Message, limit int64) (bool, int64) {
	return int64(len(message.MentionRoles)) <= limit, int64(len(message.MentionRoles))
}
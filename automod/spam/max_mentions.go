package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxMentions(message *discordgo.Message, limit int) bool {
	return len(message.Mentions) <= limit
}

func ProcessMaxRoleMentions(message *discordgo.Message, limit int) bool {
	return len(message.MentionRoles) <= limit
}
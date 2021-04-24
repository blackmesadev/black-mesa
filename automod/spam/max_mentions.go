package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxMentions(message *discordgo.Message, limit int) (bool, int) {
	return len(message.Mentions) <= limit, len(message.Mentions)
}

func ProcessMaxRoleMentions(message *discordgo.Message, limit int) (bool, int) {
	return len(message.MentionRoles) <= limit, len(message.MentionRoles)
}
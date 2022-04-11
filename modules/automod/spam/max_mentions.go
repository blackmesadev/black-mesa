package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxMentions(message *discordgo.Message, limit int64) (bool, int64, []*discordgo.User) {
	if limit == 0 {
		return true, 0, message.Mentions
	}
	return int64(len(message.Mentions)) <= limit, int64(len(message.Mentions)), message.Mentions
}

func ProcessMaxRoleMentions(message *discordgo.Message, limit int64) (bool, int64, []string) {
	if limit == 0 {
		return true, 0, message.MentionRoles
	}
	return int64(len(message.MentionRoles)) <= limit, int64(len(message.MentionRoles)), message.MentionRoles
}

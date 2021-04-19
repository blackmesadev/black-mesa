package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxAttachments(m *discordgo.Message, limit int) bool {
	return len(m.Attachments) <= limit
}
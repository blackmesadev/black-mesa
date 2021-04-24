package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxAttachments(m *discordgo.Message, limit int) (bool, int) {
	return len(m.Attachments) <= limit, len(m.Attachments)
}
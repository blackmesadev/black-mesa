package spam

import "github.com/blackmesadev/discordgo"

func ProcessMaxAttachments(m *discordgo.Message, limit int64) (bool, int64) {
	if limit == 0 {
		return true, 0
	}
	return int64(len(m.Attachments)) <= limit, int64(len(m.Attachments))
}

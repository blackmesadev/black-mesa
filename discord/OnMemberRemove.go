package discord

import (
	"time"

	"github.com/blackmesadev/black-mesa/antinuke"
	"github.com/blackmesadev/discordgo"
)

func (bot *Bot) OnMemberRemove(s *discordgo.Session, m *discordgo.GuildMemberRemove) {
	audit, err := s.GuildAuditLog(m.GuildID, m.User.ID, "", int(discordgo.AuditLogActionMemberBanAdd), 1)
	if err != nil {
		return
	}

	if len(audit.AuditLogEntries) == 0 { // wtf?
		return
	}

	antinuke.AntiRemoveProcess(audit.AuditLogEntries[0], m.GuildID, 10, time.Second)
}

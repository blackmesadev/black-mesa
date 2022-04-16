package discord

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/modules/antinuke"
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

	conf, err := db.GetConfig(m.GuildID)
	if err != nil {
		return
	}

	entry := audit.AuditLogEntries[0]
	userLevel := db.GetLevel(s, conf, m.GuildID, entry.UserID)

	anti, ok := conf.Modules.AntiNuke.MemberRemove[userLevel]

	// if theres no config for this user level, we must presume it is okay and return
	if !ok {
		return
	}

	if conf.Modules.AntiNuke.Enabled {
		ok := antinuke.AntiRemoveProcess(s, anti, entry.UserID, m.GuildID)
		if !ok {
			reason := fmt.Sprintf("AntiNuke Detection (exceeded %v/%vs)", anti.Max, anti.Interval)
			switch anti.Type {
			case "ban":
				s.GuildBanCreateWithReason(m.GuildID, entry.UserID, reason, 0)
			case "kick":
				s.GuildMemberDeleteWithReason(m.GuildID, entry.UserID, reason)
			case "rmrole":
				member, err := s.State.Member(m.GuildID, entry.UserID)
				if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
					member, err = s.GuildMember(m.GuildID, entry.UserID)
					if err != nil {
						// if we cant get the member theres nothing we can do
						return
					}
				}

				s.GuildMemberRoleBulkRemove(m.GuildID, member.User.ID, member.Roles)
			default:
			}
		}
	}
}

package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func BanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.ban") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	var permBan bool

	//idList, duration, reason := parseCommand(m.Content)
	idList := make([]string, 0)
	durationOrReasonStart := 0

	for i, possibleId := range args {
		if !userIdRegex.MatchString(possibleId) {
			durationOrReasonStart = i
			break
		}
		id := userIdRegex.FindStringSubmatch(possibleId)[1]
		idList = append(idList, id)
	}

	if len(idList) == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `ban <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := parseTime(args[durationOrReasonStart])
	reason := strings.Join(args[(durationOrReasonStart+1):], " ")

	if duration == 0 { // must be part of the reason
		permBan = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason) // append start of reason to reason
	}

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	msg := "<:mesaCheck:832350526729224243> Successfully banned "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableBan := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
		if err != nil {
			unableBan = append(unableBan, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
			AddTimedBan(m.GuildID, m.Author.ID, id, duration)

			member, err := s.State.Member(m.GuildID, id)
			if err == discordgo.ErrStateNotFound {
				member, err = s.GuildMember(m.GuildID, id)
				if err != nil {
					log.Println(err)
					unableBan = append(unableBan, id)
				} else {
					s.State.MemberAdd(member)
				}
			}
			if permBan {
				msg += "lasting `Forever` "

				logging.LogBan(s, m.GuildID, fullName, member.User, reason, m.ChannelID)
			} else {
				timeExpiry := time.Unix(duration, 0)
				timeUntil := time.Until(timeExpiry).Round(time.Second)
				msg += fmt.Sprintf("expiring `%v` (`%v`) ", timeExpiry, timeUntil.String())

				logging.LogTempBan(s, m.GuildID, fullName, member.User, time.Until(time.Unix(duration, 0)), reason, m.ChannelID)
			}
		}
	}
	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not ban %v", unableBan)
	}

	go s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v",
			time.Since(start)))
	}
}

package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func MuteCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.mute") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	var permMute bool

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
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `mute <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := parseTime(args[durationOrReasonStart])
	reason := strings.Join(args[(durationOrReasonStart+1):], " ")

	if duration == 0 { // must be part of the reason
		permMute = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason) // append start of reason to reason
	}

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	roleid := config.GetMutedRole(m.GuildID)
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	msg := "Successfully muted "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableMute := make([]string, 0)
	for _, id := range idList {
		err := s.GuildMemberRoleAdd(m.GuildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableMute = append(unableMute, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
			AddTimedRole(m.GuildID, id, roleid, duration)

			member, _ := s.State.Member(m.GuildID, id)
			if duration == 0 {
				logging.LogMute(s, m.GuildID, fullName, member.User, reason, m.ChannelID)
			} else {
				logging.LogTempMute(s, m.GuildID, fullName, member.User, time.Until(time.Unix(duration, 0)), reason, m.ChannelID)
			}
		}
	}
	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}
	if permMute {
		msg += "lasting `Forever`."

	} else {
		msg += fmt.Sprintf("expiring `%v`.", time.Unix(duration, 0))
	}

	if len(unableMute) != 0 {
		msg += fmt.Sprintf("\nCould not mute %v", unableMute)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

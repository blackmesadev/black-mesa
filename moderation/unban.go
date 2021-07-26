package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func UnbanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, PERMISSION_UNBAN) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	idList := make([]string, 0)
	durationOrReasonStart := 0

	for i, possibleId := range args {
		if !misc.UserIdRegex.MatchString(possibleId) {
			durationOrReasonStart = i
			break
		}
		id := misc.UserIdRegex.FindStringSubmatch(possibleId)[1]
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

	reason := strings.Join(args[(durationOrReasonStart+1):], " ")

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	roleid := config.GetMutedRole(m.GuildID)
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	msg := "<:mesaCheck:832350526729224243> Successfully unbanned "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableUnban := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanDeleteWithReason(m.GuildID, id, reason)
		if err != nil {
			unableUnban = append(unableUnban, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)

			user := fmt.Sprintf("`%v`", id)
			possibleUser, err := s.State.Member(m.GuildID, id)
			if err == nil {
				user = fmt.Sprintf("%v#%v (`%v`)", possibleUser.User.Username, possibleUser.User.Discriminator, possibleUser.User.ID)
			}

			logging.LogUnban(s, m.GuildID, fullName, user, reason)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableUnban) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not unban %v", unableUnban)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

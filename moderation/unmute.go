package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func UnmuteCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_UNMUTE) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	idList := make([]string, 0)
	durationOrReasonStart := 0

	for i, possibleId := range args {
		if !util.UserIdRegex.MatchString(possibleId) {
			durationOrReasonStart = i
			break
		}
		id := util.UserIdRegex.FindStringSubmatch(possibleId)[1]
		idList = append(idList, id)
	}

	if len(idList) == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `unmute <target:user[]> [time:duration] [reason:string...]`")
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

	roleid := config.GetMutedRole(m.GuildID, nil)
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	msg := "<:mesaCheck:832350526729224243> Successfully unmuted "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableUnmute := make([]string, 0)
	for _, id := range idList {

		muteInfo, err := config.GetMute(m.GuildID, id)
		if err != nil || muteInfo == nil {
			s.ChannelMessageSend(m.ChannelID, "<:mesaCheck:832350526729224243> Unable to fetch previous roles.")
		} else {
			go s.GuildMemberRoleBulkAdd(m.GuildID, id, *muteInfo.ReturnRoles)
		}

		err = s.GuildMemberRoleRemove(m.GuildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableUnmute = append(unableUnmute, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)

			possibleUser, err := s.State.Member(m.GuildID, id)
			if err != nil {
				continue
			}
			logging.LogUnmute(s, m.GuildID, fullName, possibleUser.User, reason)

			config.DeleteMute(m.GuildID, id)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableUnmute) != 0 {
		msg += fmt.Sprintf("\nCould not unmute %v", unableUnmute)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

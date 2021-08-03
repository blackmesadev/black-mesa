package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func MuteCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_KICK) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	var permMute bool

	//idList, duration, reason := parseCommand(m.Content)
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
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `mute <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := util.ParseTime(args[durationOrReasonStart])
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

	msg := "<:mesaCheck:832350526729224243> Successfully muted "

	var timeExpiry time.Time
	var timeUntil time.Duration

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableMute := make([]string, 0)
	for _, id := range idList {
		infractionUUID := uuid.New().String()

		err := s.GuildMemberRoleAdd(m.GuildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableMute = append(unableMute, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
			AddTimedMute(m.GuildID, m.Author.ID, id, roleid, duration, reason, infractionUUID)

			member, err := s.State.Member(m.GuildID, id)
			if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
				member, err = s.GuildMember(m.GuildID, id)
				if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
					log.Println(err)
					unableMute = append(unableMute, id)
				} else {
					s.State.MemberAdd(member)
				}
			}
			if member.User != nil {
				timeExpiry := time.Unix(duration, 0)
				timeUntil := time.Until(timeExpiry).Round(time.Second)
				guild, err := s.Guild(m.GuildID)
				if err == nil {
					s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, &timeExpiry, permMute, "Muted"))
				}
				if permMute {
					logging.LogMute(s, m.GuildID, fullName, member.User, reason, m.ChannelID)

				} else {
					logging.LogTempMute(s, m.GuildID, fullName, member.User, timeUntil, reason, m.ChannelID)
				}
			}
		}
	}

	if permMute {
		msg += "lasting `Forever` "
	} else {
		msg += fmt.Sprintf("expiring `%v` (`%v`) ", timeExpiry, timeUntil.String())
	}
	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableMute) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not mute %v", unableMute)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

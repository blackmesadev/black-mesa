package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func MuteCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_MUTE) {
		config.NoPermissionHandler(s, m, conf, consts.PERMISSION_MUTE)
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

	if !config.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
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

	conf, err := config.GetConfig(m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Unable to fetch Guild config.")
		return
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	roleid := conf.Modules.Moderation.MuteRole
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	var timeExpiry time.Time
	var timeUntil time.Duration

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableMute := make(map[string]error, 0)

	mutedUsers := make([]string, 0)
	updatedMutes := make([]string, 0)

	for _, id := range idList {
		infractionUUID := uuid.New().String()

		member, err := s.State.Member(m.GuildID, id)
		if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
			member, err = s.GuildMember(m.GuildID, id)
			if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
				log.Println(err)
				unableMute[id] = err
			} else {
				s.State.MemberAdd(member)
			}
		}

		res, err := AddTimedMute(m.GuildID, m.Author.ID, id, roleid, duration, reason, infractionUUID)

		if err != nil {
			unableMute[id] = err
			log.Println(err)
		} else {
			if res == MuteAlreadyMuted {
				updatedMutes = append(updatedMutes, "<@"+id+">")
			}

			if res == MuteSuccess {
				mutedUsers = append(mutedUsers, "<@"+id+">")
			}

			err := s.GuildMemberRoleAdd(m.GuildID, id, roleid) // change this to WithReason when implemented
			if err != nil {
				unableMute[id] = err
			}

			timeExpiry = time.Unix(duration, 0)
			timeUntil = time.Until(timeExpiry).Round(time.Second)

			if member.User != nil {

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

	var msg string

	if len(mutedUsers) > 0 {
		msg = "<:mesaCheck:832350526729224243> Successfully muted " + strings.Join(mutedUsers, ", ")
		if len(updatedMutes) > 0 {
			msg += " and updated the mute for " + strings.Join(updatedMutes, ", ")
		}
	} else if len(updatedMutes) > 0 {
		msg = "<:mesaCheck:832350526729224243> Successfully updated the mute for " + strings.Join(updatedMutes, ", ")
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
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not mute %v for reason `%v`", unableMute, err)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

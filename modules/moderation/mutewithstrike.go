package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func MuteWithStrikeCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, []string{consts.PERMISSION_MUTE, consts.PERMISSION_STRIKE})
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	start := time.Now()

	var permMute bool
	var permStrike bool

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
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `mws <target:user[]> [mutetime:duration] [striketime:duration] [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	reason := strings.Join(args[(durationOrReasonStart+1):], " ")
	muteDuration := util.ParseTime(args[durationOrReasonStart])

	if muteDuration == 0 { // must be part of the reason
		permMute = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason) // append start of reason to reason
	} else {
		strikeDuration := util.ParseTime(args[durationOrReasonStart+1])
		if strikeDuration == 0 {
			permStrike = true
			reason = fmt.Sprintf("%v %v", args[durationOrReasonStart+1], reason) // append start of reason to reason
		}
	}

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	roleid := conf.Modules.Moderation.MuteRole
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	var muteExpiry time.Time
	var timeUntilMuteExpiry time.Duration

	var strikeExpiry time.Time
	var timeUntilStrikeExpiry time.Duration

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableMws := make(map[string]error, 0)

	mwsUsers := make([]string, 0)
	updatedMutes := make([]string, 0)

	for _, id := range idList {
		infractionUUID := uuid.New().String()

		member, err := s.State.Member(m.GuildID, id)
		if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
			member, err = s.GuildMember(m.GuildID, id)
			if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
				log.Println(err)
				unableMws[id] = err
			} else {
				s.State.MemberAdd(member)
			}
		}

		res, err := AddTimedMute(m.GuildID, m.Author.ID, id, roleid, muteDuration, reason, infractionUUID)

		if err != nil {
			unableMws[id] = err
			log.Println(err)
		} else {
			if res == MuteAlreadyMuted {
				updatedMutes = append(updatedMutes, "<@"+id+">")
			}

			if res == MuteSuccess {
				mwsUsers = append(mwsUsers, "<@"+id+">")
			}

			err := s.GuildMemberRoleAdd(m.GuildID, id, roleid) // change this to WithReason when implemented
			if err != nil {
				unableMws[id] = err
			}

			muteExpiry = time.Unix(muteDuration, 0)
			timeUntilMuteExpiry = time.Until(muteExpiry).Round(time.Second)

			strikeExpiry = time.Unix(muteDuration, 0)
			timeUntilStrikeExpiry = time.Until(strikeExpiry).Round(time.Second)

			if member.User != nil {

				guild, err := s.Guild(m.GuildID)
				if err == nil {
					s.UserMessageSendEmbed(id, CreateMWSEmbed(member, guild, m.Author, reason, &muteExpiry, &strikeExpiry, permMute, permStrike))
				}
				logging.LogMws(s, guild.ID, fullName, member.User, timeUntilMuteExpiry, timeUntilStrikeExpiry, reason, m.ChannelID)
			}
		}
	}

	var msg string

	if len(mwsUsers) > 0 {
		msg = "<:mesaCheck:832350526729224243> Successfully muted and striked " + strings.Join(mwsUsers, ", ")
		if len(updatedMutes) > 0 {
			msg += " and updated the mute for " + strings.Join(updatedMutes, ", ")
		}
	} else if len(updatedMutes) > 0 {
		msg = "<:mesaCheck:832350526729224243> Successfully updated the mute and striked for " + strings.Join(updatedMutes, ", ")
	}

	if permMute {
		msg += " lasting `Forever` "
	} else {
		msg += fmt.Sprintf(" expiring `%v` (`%v`) ", muteExpiry, timeUntilMuteExpiry.String())
	}
	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableMws) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not mute %v users.", len(unableMws))
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

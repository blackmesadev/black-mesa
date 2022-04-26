package moderation

import (
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func BanCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_BAN)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	start := time.Now()

	var permBan bool

	var hackban bool

	var largeBan bool

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

	idLength := len(idList)

	if idLength == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `ban <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := util.ParseTime(args[durationOrReasonStart])
	reason := strings.Join(args[(durationOrReasonStart+1):], " ")

	if duration == 0 { // must be part of the reason
		permBan = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason) // append start of reason to reason
	}

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	var timeExpiry time.Time

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableBan := make(map[string]error, 0)

	largeBan = idLength > 10

	var wg sync.WaitGroup

	wg.Add(idLength)

	bannedUsers := make([]string, 0)
	updatedBans := make([]string, 0)

	for _, id := range idList {
		go func(id string) {
			defer wg.Done()

			infractionUUID := uuid.New().String()

			member, err := s.State.Member(m.GuildID, id)
			if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
				member, err = s.GuildMember(m.GuildID, id)
				if err == discordgo.ErrUnknownMember || member == nil || member.User == nil {
					hackban = true
				}
				if err != nil {
					log.Println(err)
					unableBan[id] = err
				}
			}
			timeExpiry = time.Unix(duration, 0)
			guild, err := s.Guild(m.GuildID)
			if err == nil {
				s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, &timeExpiry, permBan, "Banned"))
			}

			err = s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
			if err != nil {
				unableBan[id] = err
			} else {

				res, err := AddTimedBan(m.GuildID, m.Author.ID, id, duration, reason, infractionUUID)
				if err != nil {
					unableBan[id] = err
				}
				if !largeBan {
					if res == BanAlreadyBanned {
						updatedBans = append(updatedBans, "<@"+id+">")
					} else {
						bannedUsers = append(bannedUsers, "<@"+id+">")
					}
				}
			}

			if permBan {
				if hackban {
					logging.LogHackBan(s, m.GuildID, fullName, id, reason, m.ChannelID)
				} else {
					logging.LogBan(s, m.GuildID, fullName, member.User, reason, m.ChannelID)
				}
			} else {
				if hackban {
					logging.LogHackTempBan(s, m.GuildID, fullName, id, time.Until(time.Unix(duration, 0)), reason, m.ChannelID)
				} else {
					logging.LogTempBan(s, m.GuildID, fullName, member.User, time.Until(time.Unix(duration, 0)), reason, m.ChannelID)
				}
			}
		}(id)
	}

	wg.Wait()

	var msg string

	if len(bannedUsers) > 0 {
		msg = "<:mesaCheck:832350526729224243> Successfully banned " + strings.Join(bannedUsers, ", ")
		if len(updatedBans) > 0 {
			msg += " and updated the ban for " + strings.Join(updatedBans, ", ")
		}
	} else if len(updatedBans) > 0 {
		msg = "<:mesaCheck:832350526729224243> Successfully updated the ban for " + strings.Join(updatedBans, ", ")
	}

	if largeBan {
		msg += fmt.Sprintf("`%v` users ", idLength-len(unableBan))
	}

	if permBan {
		msg += " lasting `Forever` "
	} else {
		msg += fmt.Sprintf(" expiring <t:%v:f> (<t:%v:R>) ", timeExpiry.Unix(), timeExpiry.Unix())

	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not ban %v users.", len(unableBan))
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v",
			time.Since(start)))
	}
}

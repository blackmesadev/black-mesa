package moderation

import (
	"bytes"
	"fmt"
	"log"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func BanFileCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_BANFILE) && !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_BAN) {
		config.NoPermissionHandler(s, m, conf, consts.PERMISSION_BANFILE)
		return
	}

	start := time.Now()

	var permBan bool

	var hackban bool

	var largeBan bool

	//idList, duration, reason := parseCommand(m.Content)
	durationOrReasonStart := 0

	if len(m.Attachments) == 0 {
		return
	}

	var idList []string

	for _, file := range m.Attachments {
		if strings.HasPrefix(file.Filename, "ban") {
			resp, err := http.Get(file.URL)
			if err != nil {
				break
			}
			buf := new(bytes.Buffer)
			buf.ReadFrom(resp.Body)
			fileIDList := strings.Split(buf.String(), "\n")
			idList = fileIDList
		}
	}

	idLength := len(idList)

	if idLength == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `banfile [options:flags] [time:duration] [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	var noGuildMembers bool
	var guildMembersOnly bool
	// handle flags
	if strings.HasPrefix(args[0], "-") {
		durationOrReasonStart = 1
		for _, r := range args[0] {
			switch r {
			case 0x2d: // flag indicator
				continue
			case 0x6e: // no guild members
				noGuildMembers = true
			case 0x67: // guild members only
				guildMembersOnly = true
			default:
				s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Invalid flags.")
				return
			}
		}
	}

	// stupid flags
	if noGuildMembers && guildMembersOnly {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Invalid flags.")
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

	msg := "<:mesaCheck:832350526729224243> Successfully banned "

	var timeExpiry time.Time
	var timeUntil time.Duration

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableBan := make([]string, 0)

	largeBan = idLength > 10

	var wg sync.WaitGroup

	wg.Add(idLength)

	for _, id := range idList {
		go func(id string) {
			defer wg.Done()

			infractionUUID := uuid.New().String()

			member, err := s.State.Member(m.GuildID, id)
			if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
				member, err = s.GuildMember(m.GuildID, id)
				if err == discordgo.ErrUnknownMember || member == nil || member.User == nil {
					if guildMembersOnly {
						return
					}
					hackban = true
				}
				if member != nil && noGuildMembers {
					return
				}
				if err != nil {
					log.Println(err)
					unableBan = append(unableBan, id)
				}
			}
			timeExpiry = time.Unix(duration, 0)
			timeUntil = time.Until(timeExpiry).Round(time.Second)
			guild, err := s.Guild(m.GuildID)
			if err == nil {
				s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, &timeExpiry, permBan, "Banned"))
			}

			err = s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
			if err != nil {
				unableBan = append(unableBan, id)
			} else {
				if !largeBan {
					msg += fmt.Sprintf("<@%v> ", id)
				}
				AddTimedBan(m.GuildID, m.Author.ID, id, duration, reason, infractionUUID)
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

	if largeBan {
		msg += fmt.Sprintf("`%v` users ", idLength-len(unableBan))
	}

	if permBan {
		msg += "lasting `Forever` "
	} else {
		msg += fmt.Sprintf("expiring `%v` (`%v`) ", timeExpiry, timeUntil.String())

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

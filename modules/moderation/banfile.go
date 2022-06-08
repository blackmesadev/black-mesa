package moderation

import (
	"bytes"
	"fmt"
	"log"
	"net/http"
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

func BanFileCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, []string{consts.PERMISSION_BAN, consts.PERMISSION_BANFILE})
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	start := time.Now()

	var hackban bool

	var largeBan bool

	//idList, duration, reason := parseCommand(m.Content)

	if len(m.Attachments) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You must attach a file.")
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

	if idLength == 0 { // if there's no ids
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `banfile [options:flags]`")
		return
	}

	var noGuildMembers bool
	var guildMembersOnly bool
	// handle flags
	if len(args) > 0 {
		if strings.HasPrefix(args[0], "-") {
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
	}

	// stupid flags
	if noGuildMembers && guildMembersOnly {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Invalid flags.")
		return
	}

	msg := "<:mesaCheck:832350526729224243> Successfully banned "

	reason := "You were on the Ban List 🗒️"

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
			guild, err := s.Guild(m.GuildID)
			if err == nil {
				s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, nil, true, "Banned"))
			}

			if hackban {
				logging.LogHackBan(s, m.GuildID, fullName, id, reason, m.ChannelID)
			} else {
				logging.LogBan(s, m.GuildID, fullName, member.User, reason, m.ChannelID)
			}

			err = s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
			if err != nil {
				unableBan = append(unableBan, id)
			} else {
				if !largeBan {
					msg += fmt.Sprintf("<@%v> ", id)
				}
				AddTimedBan(m.GuildID, m.Author.ID, id, 0, reason, infractionUUID)
			}

		}(id)
	}

	wg.Wait()

	if largeBan {
		msg += fmt.Sprintf("`%v` users ", idLength-len(unableBan))
	}

	msg += "lasting `Forever` "

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v",
			time.Since(start)))
	}
}

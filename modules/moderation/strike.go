package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func StrikeCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_STRIKE)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	var reason string

	var permStrike bool

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

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `strike <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := util.ParseTime(args[durationOrReasonStart])
	reason = strings.Join(args[(durationOrReasonStart+1):], " ")

	if duration == 0 {
		permStrike = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason)
	}

	if durationOrReasonStart == 0 {
		reason = ""
	}

	reason = strings.TrimSpace(reason)

	msg := "<:mesaCheck:832350526729224243> Successfully striked "

	var timeExpiry time.Time

	unableStrike := make([]string, 0)
	for _, id := range idList {

		msg += fmt.Sprintf("<@%v> ", id)
		err := IssueStrike(s, m.GuildID, id, m.Author.ID, 1, reason, duration, m.ChannelID)

		timeExpiry = time.Unix(duration, 0)

		guild, err := s.Guild(m.GuildID)
		member, err := s.GuildMember(m.GuildID, id)
		if err == nil {
			s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, &timeExpiry, permStrike, "Striked"))
		}
		if err != nil {
			log.Println(err)
			unableStrike = append(unableStrike, id)
		}

	}

	if permStrike {
		msg += "lasting `Forever` "

	} else {

		msg += fmt.Sprintf("expiring <t:%v:f> (<t:%v:R>) ", timeExpiry.Unix(), timeExpiry.Unix())
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableStrike) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not strike %v", unableStrike)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v",
			time.Since(start)))
	}
}

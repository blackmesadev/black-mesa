package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func StrikeCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.kick") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	var reason string

	var permStrike bool

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

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `strike <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := misc.ParseTime(args[durationOrReasonStart])
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

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableStrike := make([]string, 0)
	for _, id := range idList {

		err := IssueStrike(s, m.GuildID, id, m.Author.ID, 1, reason, duration, m.ChannelID)
		if err != nil {
			unableStrike = append(unableStrike, id)
		} else {
			user, err := s.User(id)
			if err != nil {
				log.Println(err)
			} else {
				logging.LogStrike(s, m.GuildID, fullName, user, 1, reason, m.ChannelID)
			}
		}
	}

	if permStrike {
		msg += "lasting `Forever` "

	} else {
		timeExpiry := time.Unix(duration, 0)
		timeUntil := time.Until(timeExpiry).Round(time.Second)
		msg += fmt.Sprintf("expiring `%v` (`%v`) ", timeExpiry, timeUntil.String())
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableStrike) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not strike %v", unableStrike)
	}

	go s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v",
			time.Since(start)))
	}
}

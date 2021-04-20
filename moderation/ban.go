package moderation

import (
	"fmt"
	"regexp"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

var userIdRegex = regexp.MustCompile(`^(?:<@!?)?\d+>?$`)

func BanCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {

	start := time.Now()

	var permBan bool

	//idList, duration, reason := parseCommand(m.Content)
	idList := make([]string, 0)
	durationOrReasonStart := 0

	for i, possibleId := range args {
		if !userIdRegex.MatchString(possibleId) {
			durationOrReasonStart = i
			break
		}
		id := userIdRegex.FindStringSubmatch(possibleId)[0]
		idList = append(idList, id)
	}

	if len(idList) == 0 || durationOrReasonStart == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `ban <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	duration, durationErr := time.ParseDuration(args[durationOrReasonStart])
	reason := strings.Join(args[(durationOrReasonStart + 1):], " ")

	if durationErr != nil { // must be part of the reason
		permBan = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason) // append start of reason to reason
	}

	parse := time.Since(start)

	msg := "<:mesaCheck:832350526729224243> Successfully banned "

	dstart := time.Now()
	unableBan := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanCreateWithReason(m.GuildID, id, reason, 0)
		if err != nil {
			unableBan = append(unableBan, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
		}
	}
	discord := time.Since(dstart)
	msgs := time.Now()
	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if permBan {
		msg += "lasting `Forever`."

	} else {
		msg += fmt.Sprintf("expiring `%v`.", time.Unix(time.Now().Add(duration).Unix(), 0))
	}

	if len(unableBan) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not ban %v", unableBan)
	}

	msgsTotal := time.Since(msgs)
	go s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v (%v parsing, %v discordapi, %v message creation)",
			time.Since(start), parse, discord, msgsTotal))
	}
}

package moderation

import (
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func UnbanCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_UNBAN)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
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

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `unban <target:user[]> [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	reason := strings.Join(args[(durationOrReasonStart):], " ")

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	msg := "<:mesaCheck:832350526729224243> Successfully unbanned "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableUnban := make([]string, 0)
	for _, id := range idList {
		err := s.GuildBanDeleteWithReason(m.GuildID, id, reason)
		if err != nil {
			unableUnban = append(unableUnban, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)

			user := fmt.Sprintf("`%v`", id)
			possibleUser, err := s.State.Member(m.GuildID, id)
			if err == nil {
				user = fmt.Sprintf("%v#%v (`%v`)", possibleUser.User.Username, possibleUser.User.Discriminator, possibleUser.User.ID)
			}

			logging.LogUnban(s, m.GuildID, fullName, user, reason)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableUnban) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not unban %v", unableUnban)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

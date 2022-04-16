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
	"go.mongodb.org/mongo-driver/mongo"
)

func UnmuteCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_UNMUTE) {
		db.NoPermissionHandler(s, m, conf, consts.PERMISSION_UNMUTE)
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

	if len(idList) == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, consts.EMOJI_COMMAND+" `unmute <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, consts.EMOJI_CROSS+" You can not target one or more of these users.")
		return
	}

	reason := strings.Join(args[(durationOrReasonStart+1):], " ")

	if durationOrReasonStart == 0 { // fixes broken reasons
		reason = ""
	}

	reason = strings.TrimSpace(reason) // trim reason to remove random spaces

	roleid := conf.Modules.Moderation.MuteRole
	if roleid == "" {
		s.ChannelMessageSend(m.ChannelID, "Invalid Muted role ID, Aborting.")
		return
	}

	msg := consts.EMOJI_CHECK + " Successfully unmuted "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableUnmute := make([]string, 0)
	for _, id := range idList {

		muteInfo, err := db.GetMute(m.GuildID, id)
		if err != nil || muteInfo == nil {
			if err == mongo.ErrNoDocuments || err == mongo.ErrNilDocument {
				s.ChannelMessageSend(m.ChannelID, consts.EMOJI_CROSS+" Unable to find associated mute, user is likely not muted. Attempting to unmute anyway.")
			} else {
				logging.LogError(s, m.GuildID, id, "unmute", err)
			}
		}
		err = s.GuildMemberRoleRemove(m.GuildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableUnmute = append(unableUnmute, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)

			possibleUser, err := s.State.Member(m.GuildID, id)
			if err != nil {
				continue
			}
			logging.LogUnmute(s, m.GuildID, fullName, possibleUser.User, reason)

			db.DeleteMute(m.GuildID, id)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableUnmute) != 0 {
		msg += fmt.Sprintf("\nCould not unmute %v", unableUnmute)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

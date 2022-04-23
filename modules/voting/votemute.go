package voting

import (
	"context"
	"encoding/base64"
	"fmt"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func StartVoteMuteCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_VOTEMUTE)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	voting := conf.Modules.Voting.VoteMute

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
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `votemute <target:user[]> [time:duration] [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	duration := util.ParseTime(args[durationOrReasonStart])
	reason := strings.Join(args[(durationOrReasonStart+1):], " ")

	if duration == 0 { // must be part of the reason
		permMute = true
		reason = fmt.Sprintf("%v %v", args[durationOrReasonStart], reason) // append start of reason to reason
	}

	timeExpiry := time.Unix(duration, 0)
	timeUntil := time.Until(timeExpiry).Round(time.Second)

	if (int64(timeUntil.Seconds()) > voting.MaxDuration) || (voting.MaxDuration > 0 && permMute) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCross:832350526414127195> The Max Duration is %vs", voting.MaxDuration))
		return
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

	r := redis.GetRedis()

	for _, id := range idList {
		// send a message in the channel to let users know of the new vote
		msg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCheck:832350526729224243> <@%v> has started a vote to mute <@%v> for %v with reason %v",
			m.Author.ID, id, duration, reason))

		if err != nil {
			return
		}

		s.MessageReactionAdd(m.ChannelID, msg.ID, conf.Modules.Voting.UpvoteEmoji)

		// encode the reason, duration and how many current votes in base64
		reason = strings.Trim(reason, "|")
		encoded := base64.StdEncoding.EncodeToString([]byte(fmt.Sprintf("%v|%v|%v|%v", id, reason, duration, 0)))
		r.Set(context.Background(), fmt.Sprintf("vote:mute:%v:%v", m.GuildID, msg.ID), encoded, time.Until(time.Now().Add(time.Duration(voting.ExpiresAfter))))
	}
}

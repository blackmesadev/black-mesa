package discord

import (
	"context"
	"encoding/base64"
	"fmt"
	"strconv"
	"strings"

	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/modules/voting"
	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

func (bot *Bot) OnReactionAdd(s *discordgo.Session, ra *discordgo.MessageReactionAdd) {
	conf, err := db.GetConfig(ra.GuildID)
	if err != nil || conf == nil {
		conf = nil
	}

	if ra.Emoji.ID != conf.Modules.Voting.UpvoteEmoji {
		return // we dont care
	}

	// check redis for a vote
	r := bmRedis.GetRedis()
	v := r.Get(r.Context(), fmt.Sprintf("vote:mute:%v:%v", ra.GuildID, ra.MessageID))
	if v.Err() == redis.Nil {
		return
	}
	// decode vote from b64 string
	bytes, err := base64.StdEncoding.DecodeString(v.Val())
	if err != nil {
		return
	}
	vote := string(bytes)

	data := strings.Split(vote, "|")
	if len(data) != 4 { // something has gone very wrong
		return
	}

	id := data[0]
	reason := data[1]
	duration, err := strconv.Atoi(data[2])
	if err != nil {
		return
	}
	issuer := data[3]
	count, err := strconv.Atoi(data[4])
	if err != nil {
		return
	}

	count++

	// check if we have reached the threshold
	if count >= int(conf.Modules.Voting.VoteMute.UpvotesRequired) {
		// mute
		s.ChannelMessageSend(ra.ChannelID, fmt.Sprintf("<:mesaCheck:832350526790989898> VOTE PASSED: Muted <@%v> for %v for `%v`", id, duration, reason))
		voting.CompleteMute(s, conf, issuer, ra.GuildID, ra.ChannelID, id, reason, int64(duration))
		return
	}

	encoded := base64.StdEncoding.EncodeToString([]byte(fmt.Sprintf("%v|%v|%v|%v", id, reason, duration, count)))
	r.Set(context.Background(), fmt.Sprintf("vote:mute:%v:%v", ra.GuildID, ra.MessageID), encoded, 0)

}

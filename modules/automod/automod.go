package automod

import (
	"fmt"
	"log"
	"reflect"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/modules/automod/censor"
	"github.com/blackmesadev/black-mesa/modules/automod/spam"
	"github.com/blackmesadev/black-mesa/modules/moderation"
	bmRedis "github.com/blackmesadev/black-mesa/redis"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/go-redis/redis/v8"
)

var chillax = make(map[string]map[string]int64) // chllax[guildId][userId] -> exemptions remaining

var r *redis.Client

func clearCushioning(guildId string, userId string) {
	lastStrikes := chillax[guildId][userId]
	go func() {
		timer := time.NewTimer(1 * time.Minute)
		<-timer.C

		if chillax[guildId][userId] == lastStrikes {
			chillax[guildId][userId] = 0
		}
	}()
}

func addExemptMessage(guildId string, messageId string) bool {
	if r == nil {
		r = bmRedis.GetRedis()
	}

	key := fmt.Sprintf("exemptmessages:%v", guildId)
	set := r.HSet(r.Context(), key, messageId, 1)
	result, err := set.Result()
	if err != nil {
		return false
	}
	if result == 1 {
		return true
	}
	return false
}

func makeCompleteCensorStruct(automod *structs.Automod, channelID string, userLevel int64) (combined *structs.Censor) {
	censorChannel := automod.CensorChannels[channelID]

	i := 0
	automodCensorLevels := make([]int64, len(automod.CensorLevels))
	for k := range automod.CensorLevels {
		automodCensorLevels[i] = k
		i++
	}

	censorStruct := automod.CensorLevels[util.GetClosestLevel(automodCensorLevels, userLevel)]

	if censorChannel == nil {
		return censorStruct
	}

	combined = censorChannel

	cv := reflect.ValueOf(combined).Elem()
	lv := reflect.ValueOf(censorStruct).Elem()

	for i := 0; i < cv.NumField(); i++ {
		cvf := cv.Field(i)
		// Do not include boolean in combination as we should be prioritising the channel settings anyway and IsZero() will return true for a false bool
		if cvf.IsZero() && cvf.Kind() != reflect.Bool {
			cvf.Set(lv.Field(i))
		}
	}
	return combined
}

func makeCompleteSpamStruct(automod *structs.Automod, channelID string, userLevel int64) (combined *structs.Spam) {
	spamChannel := automod.SpamChannels[channelID]

	i := 0
	automodSpamLevels := make([]int64, len(automod.SpamLevels))
	for k := range automod.SpamLevels {
		automodSpamLevels[i] = k
		i++
	}

	spamLevel := automod.SpamLevels[util.GetClosestLevel(automodSpamLevels, userLevel)]

	if spamChannel == nil {
		return spamLevel
	}

	combined = spamChannel

	cv := reflect.ValueOf(combined).Elem()
	lv := reflect.ValueOf(spamLevel).Elem()

	for i := 0; i < cv.NumField(); i++ {
		cvf := cv.Field(i)
		// Do not include boolean in combination as we should be prioritising the channel settings and IsZero() will return true for a false bool
		if cvf.IsZero() && cvf.Kind() != reflect.Bool {
			cvf.Set(lv.Field(i))
		}
	}
	return combined
}

func Process(s *discordgo.Session, m *discordgo.Message) {
	conf, err := config.GetConfig(m.GuildID)

	if conf == nil || err != nil {
		return
	}

	ok, reason, weight, _ := Check(s, m, conf)
	if !ok {
		ok := addExemptMessage(m.GuildID, m.ID)
		if !ok {
			log.Printf("addExemptMessage failed on %v, %v", m.GuildID, m.ID)
		}
		go RemoveMessage(s, m)                        // remove
		if strings.HasPrefix(reason, consts.CENSOR) { // log
			logging.LogMessageCensor(s, m, reason)
		} else {
			logging.LogMessageViolation(s, m, reason)
		}

		if strings.HasPrefix(reason, consts.SPAM_MESSAGES) {
			// add a ratelimit on striking (if someone spams hard in one incident they should only receive a mute instead of being
			// escalated to a ban due to automod delay)
			if _, ok := chillax[m.GuildID]; !ok {
				chillax[m.GuildID] = make(map[string]int64)
			}

			if _, ok := chillax[m.GuildID][m.Author.ID]; !ok {
				chillax[m.GuildID][m.Author.ID] = 0
			}

			if chillax[m.GuildID][m.Author.ID] > 0 {
				chillax[m.GuildID][m.Author.ID] -= weight
				clearCushioning(m.GuildID, m.Author.ID)
				return
			}

			chillax[m.GuildID][m.Author.ID] = conf.Modules.Moderation.StrikeCushioning
			clearCushioning(m.GuildID, m.Author.ID)
		}

		err := moderation.IssueStrike(s, m.GuildID, m.Author.ID, s.State.User.ID, weight, reason, 0, m.ChannelID) // strike
		if err != nil {
			log.Println("strikes failed", err)
		}
		// and with that the moderation cycle is complete! :)
	}
}

// Return true if all is okay, return false if not.
// This function should be "silent" if a message is okay.
func Check(s *discordgo.Session, m *discordgo.Message, conf *structs.Config) (bool, string, int64, time.Time) {
	filterProcessingStart := time.Now()

	automod := conf.Modules.Automod

	content := clean(m.Content)
	lowerContent := strings.ToLower(content)

	if len(automod.SpamLevels) == 0 && len(automod.SpamChannels) == 0 &&
		len(automod.CensorLevels) == 0 && len(automod.CensorChannels) == 0 {
		return true, "", 0, filterProcessingStart
	}

	userLevel := config.GetLevel(s, conf, m.GuildID, m.Author.ID)

	// staff bypass
	if userLevel >= conf.Modules.Guild.StaffLevel && conf.Modules.Automod.StaffBypass {
		return true, "", 0, filterProcessingStart
	}

	censorStruct := makeCompleteCensorStruct(automod, m.ChannelID, userLevel)

	spamStruct := makeCompleteSpamStruct(automod, m.ChannelID, userLevel)

	// Level censors
	if censorStruct != nil {
		// Zalgo
		//if censorStruct.FilterZalgo {
		//	ok := censor.ZalgoCheck(content)
		//	if !ok {
		//		return false, "Censor->Zalgo", 1, filterProcessingStart
		//	}
		//}

		// Invites
		if censorStruct.FilterInvites {
			ok, invite := censor.InvitesWhitelistCheck(content, censorStruct.InvitesWhitelist)
			if !ok {
				return false, consts.CENSOR_INVITES + fmt.Sprintf(" (%v)", invite), 1, filterProcessingStart
			}
		} else if len(censorStruct.InvitesBlacklist) != 0 {
			ok, invite := censor.InvitesBlacklistCheck(content, censorStruct.InvitesBlacklist)
			if !ok {
				return false, consts.CENSOR_INVITES_BLACKLISTED + fmt.Sprintf(" (%v)", invite), 1, filterProcessingStart
			}
		}

		// Domains

		if censorStruct.FilterDomains {
			ok, domain := censor.DomainsWhitelistCheck(content, censorStruct.DomainWhitelist)
			if !ok {
				return false, consts.CENSOR_DOMAINS + fmt.Sprintf(" (%v)", domain), 1, filterProcessingStart
			}
		} else if len(censorStruct.DomainBlacklist) != 0 {
			ok, domain := censor.DomainsBlacklistCheck(content, censorStruct.DomainBlacklist)
			if !ok {
				return false, consts.CENSOR_DOMAINS_BLACKLISTED + fmt.Sprintf(" (%v)", domain), 1, filterProcessingStart
			}
		}

		// Strings / Substrings

		if censorStruct.FilterStrings {
			var contentList []string
			if len(m.Attachments) > 0 {
				for _, attachment := range m.Attachments {
					contentList = append(contentList, strings.ToLower(attachment.Filename))
				}
			}
			contentList = append(contentList, lowerContent)
			for _, c := range contentList {
				ok, str := censor.StringsCheck(c, censorStruct.BlockedStrings)
				if !ok {
					return false, consts.CENSOR_STRINGS + fmt.Sprintf(" (%v)", str), 1, filterProcessingStart
				}

				ok, str = censor.SubStringsCheck(c, censorStruct.BlockedSubstrings)
				if !ok {
					return false, consts.CENSOR_SUBSTRINGS + fmt.Sprintf(" (%v)", str), 1, filterProcessingStart
				}
			}
		}

		// IPs
		if censorStruct.FilterIPs {
			ok := censor.IPCheck(content)
			if !ok {
				return false, consts.CENSOR_IP, 1, filterProcessingStart
			}

		}

		//Non english characters
		if censorStruct.FilterEnglish {
			ok := censor.ExtendedUnicodeCheck(content)
			if !ok {
				return false, consts.CENSOR_NOTENGLISH, 1, filterProcessingStart
			}
		}

		// Obnoxious Unicode
		if censorStruct.FilterObnoxiousUnicode {
			ok := censor.ObnoxiousUnicodeCheck(content)
			if !ok {
				return false, consts.CENSOR_OBNOXIOUSUNICODE, 1, filterProcessingStart
			}
		}

		// Regex
		if censorStruct.FilterRegex {
			matches, ok := censor.RegexCheck(content, censorStruct.Regex)
			if !ok {
				return false, fmt.Sprintf("%v (%v)", consts.CENSOR_REGEX, matches), 1, filterProcessingStart
			}
		}
	}

	// Level Spam
	if spamStruct != nil {

		// Messages
		interval := time.Duration(spamStruct.Interval) * time.Second
		ok := spam.ProcessMaxMessages(m.Author.ID, m.GuildID, spamStruct.MaxMessages, interval, false)
		if !ok {
			err := spam.ClearMaxMessages(m.Author.ID, m.GuildID)
			if err != nil {
				logging.LogError(s, m.GuildID, m.Author.ID, "spam.ClearMaxMessages", err)
			}
			return false, consts.SPAM_MESSAGES + fmt.Sprintf(" (%v/%v)", spamStruct.MaxMessages, interval.String()), 1, filterProcessingStart
		}

		// newlines

		ok, count := spam.ProcessMaxNewlines(m.Content, spamStruct.MaxNewlines)
		if !ok {
			return false, consts.SPAM_NEWLINES + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxNewlines), 1, filterProcessingStart
		}

		// mentions
		var mentions []*discordgo.User
		ok, count, mentions = spam.ProcessMaxMentions(m, spamStruct.MaxMentions)
		if !ok {
			alertMentionedUsers(s, m.GuildID, mentions)
			return false, consts.SPAM_MENTIONS + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxMentions), 1, filterProcessingStart
		}
		//var roleMentions []string
		ok, count, _ = spam.ProcessMaxRoleMentions(m, spamStruct.MaxMentions)
		if !ok {
			//alertMentionedRoles(s, m.GuildID, roleMentions)
			return false, consts.SPAM_MENTIONS + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxMentions), 1, filterProcessingStart
		}

		// links

		ok, count = spam.ProcessMaxLinks(m.Content, spamStruct.MaxLinks)
		if !ok {
			return false, consts.SPAM_LINKS + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxLinks), 1, filterProcessingStart
		}

		// uppercase
		ok, percent := spam.ProcessMaxUppercase(m.Content, spamStruct.MaxUppercasePercent, int(spamStruct.MinUppercaseLimit))
		if !ok {
			return false, consts.SPAM_UPPERCASE + fmt.Sprintf(" (%v%% > %v%%)", percent, spamStruct.MaxUppercasePercent), 1, filterProcessingStart
		}
		// emoji

		ok, count = spam.ProcessMaxEmojis(m, spamStruct.MaxEmojis)
		if !ok {
			return false, consts.SPAM_EMOJIS + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxEmojis), 1, filterProcessingStart
		}

		// attachments

		ok, count = spam.ProcessMaxAttachments(m, spamStruct.MaxAttachments)
		if !ok {
			return false, consts.SPAM_ATTACHMENTS + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxAttachments), 1, filterProcessingStart
		}

		// length

		ok, count = spam.ProcessMaxLength(m, spamStruct.MaxCharacters)
		if !ok {
			return false, consts.SPAM_MAXLENTH + fmt.Sprintf(" (%v > %v)", count, spamStruct.MaxCharacters), 1, filterProcessingStart

		}
	}

	return true, "", 0, filterProcessingStart
}

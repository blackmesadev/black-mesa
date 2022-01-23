package automod

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/automod/censor"
	"github.com/blackmesadev/black-mesa/automod/spam"
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/moderation"
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

	if len(automod.SpamLevels) == 0 && len(automod.SpamChannels) == 0 &&
		len(automod.CensorLevels) == 0 && len(automod.SpamChannels) == 0 {
		return true, "", 0, filterProcessingStart
	}

	userLevel := config.GetLevel(s, m.GuildID, m.Author.ID)

	censorChannel := automod.CensorChannels[m.ChannelID]
	spamChannel := automod.SpamChannels[m.ChannelID]

	i := 0
	automodCensorLevels := make([]int64, len(automod.CensorLevels))
	for k := range automod.CensorLevels {
		automodCensorLevels[i] = k
		i++
	}

	i = 0
	automodSpamLevels := make([]int64, len(automod.SpamLevels))
	for k := range automod.SpamLevels {
		automodSpamLevels[i] = k
		i++
	}

	censorLevel := automod.CensorLevels[util.GetClosestLevel(automodCensorLevels, userLevel)]
	spamLevel := automod.SpamLevels[util.GetClosestLevel(automodSpamLevels, userLevel)]

	// !!channels take priority!!

	// Channel censors
	if censorChannel != nil {
		// Zalgo
		//if censorChannel.FilterZalgo {
		//	ok := censor.ZalgoCheck(content)
		//	if !ok {
		//		return false, consts.CENSOR_ZALGO, 1, filterProcessingStart
		//	}
		//}

		// Invites
		if censorChannel.FilterInvites {
			ok, invite := censor.InvitesWhitelistCheck(content, censorChannel.InvitesWhitelist)
			if !ok {
				return false, consts.CENSOR_INVITES + fmt.Sprintf(" (%v)", invite), 1, filterProcessingStart
			}

		} else if len(*censorChannel.InvitesBlacklist) != 0 {
			ok, invite := censor.InvitesBlacklistCheck(content, censorChannel.InvitesBlacklist)
			if !ok {
				return false, consts.CENSOR_INVITES_BLACKLISTED + fmt.Sprintf(" (%v)", invite), 1, filterProcessingStart
			}
		}

		// Domains

		if censorChannel.FilterDomains {
			ok, domain := censor.DomainsWhitelistCheck(content, censorChannel.DomainWhitelist)
			if !ok {
				return false, consts.CENSOR_DOMAINS + fmt.Sprintf(" (%v)", domain), 1, filterProcessingStart
			}
		} else if len(*censorChannel.DomainBlacklist) != 0 {
			ok, domain := censor.DomainsBlacklistCheck(content, censorChannel.DomainBlacklist)
			if !ok {
				return false, consts.CENSOR_DOMAINS_BLACKLISTED + fmt.Sprintf(" (%v)", domain), 1, filterProcessingStart
			}
		}

		// Strings / Substrings

		if censorChannel.FilterStrings {
			ok, str := censor.StringsCheck(replaceNonStandardSpace(m.Content), censorChannel.BlockedStrings)
			if !ok {
				return false, consts.CENSOR_STRINGS + fmt.Sprintf(" (%v)", str), 1, filterProcessingStart
			}

			ok, str = censor.SubStringsCheck(content, censorChannel.BlockedSubstrings)
			if !ok {
				return false, consts.CENSOR_SUBSTRINGS + fmt.Sprintf(" (%v)", str), 1, filterProcessingStart
			}
		}

		// IPs
		if censorChannel.FilterIPs {
			ok := censor.IPCheck(content)
			if !ok {
				return false, consts.CENSOR_IP, 1, filterProcessingStart
			}

		}

		// Obnoxious Unicode
		if censorChannel.FilterObnoxiousUnicode {
			ok := censor.ObnoxiousUnicodeCheck(content)
			if !ok {
				return false, consts.CENSOR_OBNOXIOUSUNICODE, 1, filterProcessingStart
			}
		}

		//Non english characters
		if censorChannel.FilterEnglish {
			ok := censor.ExtendedUnicodeCheck(content)
			if !ok {
				return false, consts.CENSOR_NOTENGLISH, 1, filterProcessingStart
			}
		}

		// Regex
		if censorChannel.FilterRegex {
			matches, ok := censor.RegexCheck(content, censorChannel.Regex)
			if !ok {
				return false, fmt.Sprintf("%v (`%v`)", consts.CENSOR_REGEX, matches), 1, filterProcessingStart
			}
		}
	}

	// Level censors
	if censorLevel != nil && censorChannel == nil {
		// Zalgo
		//if censorLevel.FilterZalgo {
		//	ok := censor.ZalgoCheck(content)
		//	if !ok {
		//		return false, "Censor->Zalgo", 1, filterProcessingStart
		//	}
		//}

		// Invites
		if censorLevel.FilterInvites {
			ok, invite := censor.InvitesWhitelistCheck(content, censorLevel.InvitesWhitelist)
			if !ok {
				return false, consts.CENSOR_INVITES + fmt.Sprintf(" (%v)", invite), 1, filterProcessingStart
			}
		} else if len(*censorLevel.InvitesBlacklist) != 0 {
			ok, invite := censor.InvitesBlacklistCheck(content, censorLevel.InvitesBlacklist)
			if !ok {
				return false, consts.CENSOR_INVITES_BLACKLISTED + fmt.Sprintf(" (%v)", invite), 1, filterProcessingStart
			}
		}

		// Domains

		if censorLevel.FilterDomains {
			ok, domain := censor.DomainsWhitelistCheck(content, censorLevel.DomainWhitelist)
			if !ok {
				return false, consts.CENSOR_DOMAINS + fmt.Sprintf(" (%v)", domain), 1, filterProcessingStart
			}
		} else if len(*censorLevel.DomainBlacklist) != 0 {
			ok, domain := censor.DomainsBlacklistCheck(content, censorLevel.DomainBlacklist)
			if !ok {
				return false, consts.CENSOR_DOMAINS_BLACKLISTED + fmt.Sprintf(" (%v)", domain), 1, filterProcessingStart
			}
		}

		// Strings / Substrings

		if censorLevel.FilterStrings {
			var contentList []string
			if len(m.Attachments) > 0 {
				for _, attachment := range m.Attachments {
					contentList = append(contentList, attachment.Filename)
				}
			}
			contentList = append(contentList, content)
			for _, content := range contentList {
				ok, str := censor.StringsCheck(content, censorLevel.BlockedStrings)
				if !ok {
					return false, consts.CENSOR_STRINGS + fmt.Sprintf(" (%v)", str), 1, filterProcessingStart
				}

				ok, str = censor.SubStringsCheck(content, censorLevel.BlockedSubstrings)
				if !ok {
					return false, consts.CENSOR_SUBSTRINGS + fmt.Sprintf(" (%v)", str), 1, filterProcessingStart
				}
			}
		}

		// IPs
		if censorLevel.FilterIPs {
			ok := censor.IPCheck(content)
			if !ok {
				return false, consts.CENSOR_IP, 1, filterProcessingStart
			}

		}

		//Non english characters
		if censorLevel.FilterEnglish {
			ok := censor.ExtendedUnicodeCheck(content)
			if !ok {
				return false, consts.CENSOR_NOTENGLISH, 1, filterProcessingStart
			}
		}

		// Obnoxious Unicode
		if censorLevel.FilterObnoxiousUnicode {
			ok := censor.ObnoxiousUnicodeCheck(content)
			if !ok {
				return false, consts.CENSOR_OBNOXIOUSUNICODE, 1, filterProcessingStart
			}
		}

		// Regex
		if censorLevel.FilterRegex {
			matches, ok := censor.RegexCheck(content, censorLevel.Regex)
			if !ok {
				return false, fmt.Sprintf("%v (%v)", consts.CENSOR_REGEX, matches), 1, filterProcessingStart
			}
		}
	}

	// Channel Spam
	if spamChannel != nil {

		// Messages
		interval := time.Duration(spamChannel.Interval) * time.Second
		ok := spam.ProcessMaxMessages(m.Author.ID, m.GuildID, spamChannel.MaxMessages, interval, false)
		if !ok {
			err := spam.ClearMaxMessages(m.Author.ID, m.GuildID)
			if err != nil {
				logging.LogError(s, m.GuildID, m.Author.ID, "spam.ClearMaxMessages", err)
			}
			return false, consts.SPAM_MESSAGES + fmt.Sprintf(" (%v/%v)", spamChannel.MaxMessages, interval.String()), 1, filterProcessingStart
		}

		// newlines

		ok, count := spam.ProcessMaxNewlines(m.Content, spamChannel.MaxNewlines)
		if !ok {
			return false, consts.SPAM_NEWLINES + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxNewlines), 1, filterProcessingStart
		}

		// mentions
		var mentions []*discordgo.User
		ok, count, mentions = spam.ProcessMaxMentions(m, spamChannel.MaxMentions)
		if !ok {
			alertMentionedUsers(s, m.GuildID, mentions)
			return false, consts.SPAM_MENTIONS + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxMentions), 1, filterProcessingStart
		}
		//var roleMentions []string
		ok, count, _ = spam.ProcessMaxRoleMentions(m, spamChannel.MaxMentions)
		if !ok {
			//alertMentionedRoles(s, m.GuildID, roleMentions) : TODO - find a way to
			return false, consts.SPAM_MENTIONS + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxMentions), 1, filterProcessingStart
		}

		// links

		ok, count = spam.ProcessMaxLinks(m.Content, spamChannel.MaxLinks)
		if !ok {
			return false, consts.SPAM_LINKS + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxLinks), 1, filterProcessingStart
		}

		// uppercase
		ok, percent := spam.ProcessMaxUppercase(m.Content, spamChannel.MaxUppercasePercent, int(spamChannel.MinUppercaseLimit))
		if !ok {
			return false, consts.SPAM_UPPERCASE + fmt.Sprintf(" (%v%% > %v%%)", percent, spamChannel.MaxUppercasePercent), 1, filterProcessingStart
		}
		// emoji

		ok, count = spam.ProcessMaxEmojis(m, spamChannel.MaxEmojis)
		if !ok {
			return false, consts.SPAM_EMOJIS + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxEmojis), 1, filterProcessingStart
		}

		// attachments

		ok, count = spam.ProcessMaxAttachments(m, spamChannel.MaxAttachments)
		if !ok {
			return false, consts.SPAM_ATTACHMENTS + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxAttachments), 1, filterProcessingStart
		}

		// length

		ok, count = spam.ProcessMaxLength(m, spamChannel.MaxCharacters)
		if !ok {
			return false, consts.SPAM_MAXLENTH + fmt.Sprintf(" (%v > %v)", count, spamChannel.MaxCharacters), 1, filterProcessingStart

		}
	}

	// Level Spam
	if spamLevel != nil && spamChannel == nil {

		// Messages
		interval := time.Duration(spamLevel.Interval) * time.Second
		ok := spam.ProcessMaxMessages(m.Author.ID, m.GuildID, spamLevel.MaxMessages, interval, false)
		if !ok {
			err := spam.ClearMaxMessages(m.Author.ID, m.GuildID)
			if err != nil {
				logging.LogError(s, m.GuildID, m.Author.ID, "spam.ClearMaxMessages", err)
			}
			return false, consts.SPAM_MESSAGES + fmt.Sprintf(" (%v/%v)", spamLevel.MaxMessages, interval.String()), 1, filterProcessingStart
		}

		// newlines

		ok, count := spam.ProcessMaxNewlines(m.Content, spamLevel.MaxNewlines)
		if !ok {
			return false, consts.SPAM_NEWLINES + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxNewlines), 1, filterProcessingStart
		}

		// mentions
		var mentions []*discordgo.User
		ok, count, mentions = spam.ProcessMaxMentions(m, spamLevel.MaxMentions)
		if !ok {
			alertMentionedUsers(s, m.GuildID, mentions)
			return false, consts.SPAM_MENTIONS + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxMentions), 1, filterProcessingStart
		}
		//var roleMentions []string
		ok, count, _ = spam.ProcessMaxRoleMentions(m, spamLevel.MaxMentions)
		if !ok {
			//alertMentionedRoles(s, m.GuildID, roleMentions)
			return false, consts.SPAM_MENTIONS + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxMentions), 1, filterProcessingStart
		}

		// links

		ok, count = spam.ProcessMaxLinks(m.Content, spamLevel.MaxLinks)
		if !ok {
			return false, consts.SPAM_LINKS + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxLinks), 1, filterProcessingStart
		}

		// uppercase
		ok, percent := spam.ProcessMaxUppercase(m.Content, spamLevel.MaxUppercasePercent, int(spamLevel.MinUppercaseLimit))
		if !ok {
			return false, consts.SPAM_UPPERCASE + fmt.Sprintf(" (%v%% > %v%%)", percent, spamLevel.MaxUppercasePercent), 1, filterProcessingStart
		}
		// emoji

		ok, count = spam.ProcessMaxEmojis(m, spamLevel.MaxEmojis)
		if !ok {
			return false, consts.SPAM_EMOJIS + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxEmojis), 1, filterProcessingStart
		}

		// attachments

		ok, count = spam.ProcessMaxAttachments(m, spamLevel.MaxAttachments)
		if !ok {
			return false, consts.SPAM_ATTACHMENTS + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxAttachments), 1, filterProcessingStart
		}

		// length

		ok, count = spam.ProcessMaxLength(m, spamLevel.MaxCharacters)
		if !ok {
			return false, consts.SPAM_MAXLENTH + fmt.Sprintf(" (%v > %v)", count, spamLevel.MaxCharacters), 1, filterProcessingStart

		}
	}

	return true, "", 0, filterProcessingStart
}

package voting

import (
	"fmt"
	"log"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/modules/moderation"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func CompleteMute(s *discordgo.Session, conf *structs.Config, issuer string, guildID string, channelID string, id string, reason string, duration int64) {
	var timeExpiry time.Time
	var timeUntil time.Duration

	unableMute := make(map[string]error, 0)

	mutedUsers := make([]string, 0)
	updatedMutes := make([]string, 0)

	infractionUUID := uuid.New().String()

	member, err := s.State.Member(guildID, id)
	if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
		member, err = s.GuildMember(guildID, id)
		if err == discordgo.ErrStateNotFound || member == nil || member.User == nil {
			log.Println(err)
			unableMute[id] = err
		} else {
			s.State.MemberAdd(member)
		}
	}

	issuerMember, err := s.State.Member(guildID, id)
	if err == discordgo.ErrStateNotFound || issuerMember == nil || issuerMember.User == nil {
		issuerMember, err = s.GuildMember(guildID, id)
		if err == discordgo.ErrStateNotFound || issuerMember == nil || issuerMember.User == nil {
			log.Println(err)
			unableMute[id] = err
		} else {
			s.State.MemberAdd(member)
		}
	}

	roleid := conf.Modules.Moderation.MuteRole
	if roleid == "" {
		s.ChannelMessageSend(channelID, fmt.Sprintf("%v VOTE MUTE: Invalid Muted role ID, Aborting.", consts.EMOJI_CROSS))
		return
	}

	fullName := issuerMember.User.Username + "#" + issuerMember.User.Discriminator

	var permMute bool
	if duration == 0 {
		permMute = true
	}

	res, err := moderation.AddTimedMute(guildID, issuer, id, roleid, duration, reason, infractionUUID)

	if err != nil {
		unableMute[id] = err
		log.Println(err)
	} else {
		if res == moderation.MuteAlreadyMuted {
			updatedMutes = append(updatedMutes, "<@"+id+">")
		}

		if res == moderation.MuteSuccess {
			mutedUsers = append(mutedUsers, "<@"+id+">")
		}

		err := s.GuildMemberRoleAdd(guildID, id, roleid) // change this to WithReason when implemented
		if err != nil {
			unableMute[id] = err
		}

		timeExpiry = time.Unix(duration, 0)
		timeUntil = time.Until(timeExpiry).Round(time.Second)

		if member.User != nil {

			guild, err := s.Guild(guildID)
			if err == nil {
				s.UserMessageSendEmbed(id, moderation.CreatePunishmentEmbed(member, guild, issuerMember.User, reason, &timeExpiry, permMute, "Muted"))
			}
			if permMute {
				logging.LogMute(s, guildID, fullName, member.User, reason, channelID)

			} else {
				logging.LogTempMute(s, guildID, fullName, member.User, timeUntil, reason, channelID)
			}
		}
	}
}

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
	"github.com/google/uuid"
)

func KickCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_KICK)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	var reason string

	start := time.Now()

	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `kick <target:user[]> [reason:string...]`")
		return
	}

	if !db.CheckTargets(s, conf, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	reasonSearch := util.SnowflakeRegex.Split(m.Content, -1)

	search := reasonSearch[len(reasonSearch)-1]

	if search != "" {
		if search[:1] == ">" {
			reason = reasonSearch[len(reasonSearch)-1][1:]
		} else {
			reason = reasonSearch[len(reasonSearch)-1]
		}
	}

	reason = strings.TrimSpace(reason)

	msg := "<:mesaCheck:832350526729224243> Successfully kicked "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableKick := make(map[string]error, 0)
	for _, id := range idList {
		infractionUUID := uuid.New().String()

		member, err := s.State.Member(m.GuildID, id)
		if err == discordgo.ErrStateNotFound {
			member, err = s.GuildMember(m.GuildID, id)
			if err != nil {
				unableKick[id] = err
			}
		}
		guild, err := s.Guild(m.GuildID)
		if err == nil {
			s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, nil, false, "Kicked"))
		}
		logging.LogKick(s, m.GuildID, fullName, member.User, reason, m.ChannelID)
		err = s.GuildMemberDeleteWithReason(m.GuildID, id, reason)

		if err != nil {
			unableKick[id] = err
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
			AddKick(m.GuildID, m.Author.ID, id, reason, infractionUUID)
		}
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableKick) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not kick %v users.", len(unableKick))
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

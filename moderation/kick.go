package moderation

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func KickCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_KICK) {
		util.NoPermissionHandler(s, m, conf, consts.PERMISSION_KICK)
		return
	}

	var reason string

	start := time.Now()

	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `kick <target:user[]> [reason:string...]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
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
	unableKick := make([]string, 0)
	for _, id := range idList {
		infractionUUID := uuid.New().String()

		member, err := s.State.Member(m.GuildID, id)
		if err == discordgo.ErrStateNotFound {
			member, err = s.GuildMember(m.GuildID, id)
			if err != nil {
				log.Println(err)
				unableKick = append(unableKick, id)
			}
		}
		guild, err := s.Guild(m.GuildID)
		if err == nil {
			s.UserMessageSendEmbed(id, CreatePunishmentEmbed(member, guild, m.Author, reason, nil, false, "Kicked"))
		}
		err = s.GuildMemberDeleteWithReason(m.GuildID, id, reason)

		if err != nil {
			unableKick = append(unableKick, id)
		} else {
			msg += fmt.Sprintf("<@%v> ", id)
			AddKick(m.GuildID, m.Author.ID, id, reason, infractionUUID)
		}
		logging.LogKick(s, m.GuildID, fullName, member.User, reason, m.ChannelID)
	}

	if len(reason) != 0 {
		msg += fmt.Sprintf("for reason `%v` ", reason)
	}

	if len(unableKick) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not kick %v", unableKick)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

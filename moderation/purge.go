package moderation

import (
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func PurgeCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_PURGE) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	argsLength := len(args)

	if argsLength < 1 || argsLength > 2 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `purge <messages:int> [type:string] [filter:string...]`")
		return
	}

	purgeType := consts.PURGE_ALL

	start := time.Now()

	msgLimitString := args[0]

	var typeString string

	if argsLength == 2 {
		typeString = args[1]
	}

	if typeString != "" {
		purgeType = strings.ToLower(typeString)
	}

	msgLimit, err := strconv.Atoi(msgLimitString)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `purge <messages:int> [type:string] [filter:string...]`")
		return
	}

	switch purgeType {
	case consts.PURGE_ALL:
		PurgeAll(s, m, msgLimit)
	case consts.PURGE_ATTACHEMENTS:
		PurgeAttachments(s, m, msgLimit)
	case consts.PURGE_BOT:
		PurgeBot(s, m, msgLimit)
	case consts.PURGE_IMAGE:
		PurgeImage(s, m, msgLimit)
	case consts.PURGE_STRING:
		if len(args) < 3 {
			s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `purge <messages:int> [type:string] [filter:string...]`")
			return
		}
		filter := strings.Join(args[2:], " ")
		PurgeString(s, m, msgLimit, filter)
	case consts.PURGE_USER:
		PurgeUser(s, m, msgLimit)
	case consts.PURGE_VIDEO:
		PurgeVideo(s, m, msgLimit)

	default:
		PurgeAll(s, m, msgLimit)
	}

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

func PurgeAttachments(s *discordgo.Session, m *discordgo.Message, msgLimit int) {

}

func PurgeBot(s *discordgo.Session, m *discordgo.Message, msgLimit int) {

}

func PurgeImage(s *discordgo.Session, m *discordgo.Message, msgLimit int) {

}

func PurgeString(s *discordgo.Session, m *discordgo.Message, msgLimit int, filter string) {

}

func PurgeUser(s *discordgo.Session, m *discordgo.Message, msgLimit int) {

}

func PurgeVideo(s *discordgo.Session, m *discordgo.Message, msgLimit int) {

}

func PurgeAll(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		util.ErrorHandler(s, m.ChannelID, err)
		return
	}

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, "", lastID, "")
		if err != nil {
			util.ErrorHandler(s, m.ChannelID, err)
			return
		}
		for _, msg := range msgList {
			lastID = msg.ID
			err := s.ChannelMessageDelete(m.ChannelID, m.ID)
			if err != nil {
				util.ErrorHandler(s, m.ChannelID, err)
				return
			}
			count++
			if count == msgLimit {
				break
			}
		}
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
}

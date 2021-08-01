package moderation

import (
	"fmt"
	"strconv"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func PurgeCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_PURGE) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	argsLength := len(args)

	if argsLength < 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `purge <messages:int> [type:string] [filter:string...]`")
		return
	}

	var purgeType consts.PurgeType

	start := time.Now()

	msgLimitString := args[0]

	var typeString string

	if argsLength >= 2 {
		typeString = args[1]
	} else {
		purgeType = consts.PURGE_ALL
	}

	if typeString != "" {
		purgeType = consts.PurgeType(strings.ToLower(typeString))
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
		filter = strings.ToLower(filter)
		PurgeString(s, m, msgLimit, filter)
	case consts.PURGE_USER:
		PurgeUser(s, m, msgLimit)
	case consts.PURGE_VIDEO:
		PurgeVideo(s, m, msgLimit)

	default:
		var filter string
		if len(args) >= 3 {
			filter = strings.Join(args[2:], " ")
			filter = strings.ToLower(filter)
		} else {
			filter = ""
		}

		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `purge <messages:int> [type:string] [filter:string...]`")
		return
	}

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

func PurgeAttachments(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			if len(msg.Attachments) > 0 {
				err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
				if err != nil {
					misc.ErrorHandler(s, m.ChannelID, err)
					return
				}
				count++
				if count == msgLimit {
					break
				}
				// Update the purge message every 5 messages it deletes.
				if count%5 == 0 {
					s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
				}
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

}

func PurgeBot(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			if msg.Author.Bot || msg.Author.System {
				err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
				if err != nil {
					misc.ErrorHandler(s, m.ChannelID, err)
					return
				}
				count++
				if count == msgLimit {
					break
				}
				// Update the purge message every 5 messages it deletes.
				if count%5 == 0 {
					s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
				}
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

}

func PurgeImage(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			if util.CheckForImage(msg) {
				err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
				if err != nil {
					misc.ErrorHandler(s, m.ChannelID, err)
					return
				}
				count++
				if count == msgLimit {
					break
				}
				// Update the purge message every 5 messages it deletes.
				if count%5 == 0 {
					s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
				}
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

}

func PurgeString(s *discordgo.Session, m *discordgo.Message, msgLimit int, filter string) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages by `%v`... [%v/%v]", filter, count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			if strings.Contains(strings.ToLower(msg.Content), filter) {
				err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
				if err != nil {
					misc.ErrorHandler(s, m.ChannelID, err)
					return
				}
				count++
				if count == msgLimit {
					break
				}
				// Update the purge message every 5 messages it deletes.
				if count%5 == 0 {
					s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages by `%v`... [%v/%v]", filter, count, msgLimit))
				}
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages by `%v`... [%v/%v]", filter, count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

}

func PurgeUser(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			if !msg.Author.Bot {
				err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
				if err != nil {
					misc.ErrorHandler(s, m.ChannelID, err)
					return
				}
				count++
				if count == msgLimit {
					break
				}
				// Update the purge message every 5 messages it deletes.
				if count%5 == 0 {
					s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
				}
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)
}

func PurgeVideo(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			if util.CheckForVideo(msg) {
				err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
				if err != nil {
					misc.ErrorHandler(s, m.ChannelID, err)
					return
				}
				count++
				if count == msgLimit {
					break
				}
				// Update the purge message every 5 messages it deletes.
				if count%5 == 0 {
					s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
				}
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)
}

func PurgeAll(s *discordgo.Session, m *discordgo.Message, msgLimit int) {
	var count int
	var lastID string

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return
	}

	lastID = progressMsg.ID // just set lastid to this so that it wont delete the purge message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return
		}
		if len(msgList) == 0 {
			break
		}
		for _, msg := range msgList {
			lastID = msg.ID
			err := s.ChannelMessageDelete(m.ChannelID, msg.ID)
			if err != nil {
				misc.ErrorHandler(s, m.ChannelID, err)
				return
			}
			count++
			if count == msgLimit {
				break
			}
			// Update the purge message every 5 messages it deletes.
			if count%5 == 0 {
				s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
			}
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)
}

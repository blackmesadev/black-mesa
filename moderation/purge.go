package moderation

import (
	"fmt"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/apiwrapper"
	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
	"github.com/google/uuid"
)

func PurgeCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_PURGE) {
		util.NoPermissionHandler(s, m, conf, consts.PERMISSION_PURGE)
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

	var allMessages []*discordgo.Message

	switch purgeType {
	case consts.PURGE_ALL:
		allMessages = PurgeAll(s, m, msgLimit)
	case consts.PURGE_ATTACHEMENTS:
		allMessages = PurgeAttachments(s, m, msgLimit)
	case consts.PURGE_BOT:
		allMessages = PurgeBot(s, m, msgLimit)
	case consts.PURGE_IMAGE:
		allMessages = PurgeImage(s, m, msgLimit)
	case consts.PURGE_STRING:
		if len(args) < 3 {
			s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `purge <messages:int> [type:string] [filter:string...]`")
			return
		}
		filter := strings.Join(args[2:], " ")
		filter = strings.ToLower(filter)
		allMessages = PurgeString(s, m, msgLimit, filter)
	case consts.PURGE_USER:
		allMessages = PurgeUser(s, m, msgLimit)
	case consts.PURGE_VIDEO:
		allMessages = PurgeVideo(s, m, msgLimit)

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

	uuidString := uuid.New().String()

	purges := &structs.PurgeStruct{
		Messages:  allMessages,
		GuildID:   m.GuildID,
		ChannelID: m.ChannelID,
		IssuerID:  m.Author.ID,
		UUID:      uuidString,
	}

	resp, err := apiwrapper.ApiInstance.SendPurges(purges)
	if err != nil || resp == nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Unable to submit purge data to database. Please contact a developer. `%v`", consts.EMOJI_CROSS, err))
	}

	if resp.StatusCode == http.StatusOK {
		logging.LogPurge(s, m, uuidString)
	}

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

func PurgeAttachments(s *discordgo.Session, m *discordgo.Message, msgLimit int) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		msgIDList := make([]string, 0)
		for _, msg := range msgList {
			lastID = msg.ID
			if len(msg.Attachments) > 0 {
				msgIDList = append(msgIDList, msg.ID)
				count++
				if count == msgLimit {
					break
				}
			}
		}
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			log.Println(err, msgIDList)
		} else {
			allMessages = append(allMessages, msgList...)
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

func PurgeBot(s *discordgo.Session, m *discordgo.Message, msgLimit int) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		msgIDList := make([]string, 0)
		for _, msg := range msgList {
			lastID = msg.ID
			if msg.Author.Bot || msg.Author.System {
				msgIDList = append(msgIDList, msg.ID)
				count++
				if count == msgLimit {
					break
				}
			}
		}
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			log.Println(err, msgIDList)
		} else {
			allMessages = append(allMessages, msgList...)
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

func PurgeImage(s *discordgo.Session, m *discordgo.Message, msgLimit int) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		msgIDList := make([]string, 0)
		for _, msg := range msgList {
			lastID = msg.ID
			if util.CheckForImage(msg) {
				msgIDList = append(msgIDList, msg.ID)
				count++
				if count == msgLimit {
					break
				}
			}
		}
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			log.Println(err, msgIDList)
		} else {
			allMessages = append(allMessages, msgList...)
		}

		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

func PurgeString(s *discordgo.Session, m *discordgo.Message, msgLimit int, filter string) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages by `%v`... [%v/%v]", filter, count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		msgIDList := make([]string, 0)
		for _, msg := range msgList {
			lastID = msg.ID
			if strings.Contains(strings.ToLower(msg.Content), filter) {
				msgIDList = append(msgIDList, msg.ID)
				count++
				if count == msgLimit {
					break
				}
			}
		}
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			log.Println(err, msgIDList)
		} else {
			allMessages = append(allMessages, msgList...)
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages by `%v`... [%v/%v]", filter, count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

func PurgeUser(s *discordgo.Session, m *discordgo.Message, msgLimit int) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		msgIDList := make([]string, 0)
		for _, msg := range msgList {
			lastID = msg.ID
			if !msg.Author.Bot {
				msgIDList = append(msgIDList, msg.ID)
				count++
				if count == msgLimit {
					break
				}
			}
		}
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			log.Println(err, msgIDList)
		} else {
			allMessages = append(allMessages, msgList...)
		}

		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

func PurgeVideo(s *discordgo.Session, m *discordgo.Message, msgLimit int) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, 100, lastID, "", "")
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		msgIDList := make([]string, 0)
		for _, msg := range msgList {
			lastID = msg.ID
			if util.CheckForVideo(msg) {
				msgIDList = append(msgIDList, msg.ID)
				count++
				if count == msgLimit {
					break
				}
			}
		}
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			log.Println(err, msgIDList)
		} else {
			allMessages = append(allMessages, msgList...)
		}
		// Update at the end with newest count before waiting and deleting
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	}
	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

func PurgeAll(s *discordgo.Session, m *discordgo.Message, msgLimit int) []*discordgo.Message {
	var count int
	var lastID string

	allMessages := make([]*discordgo.Message, 0)

	progressMsg, err := s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))
	if err != nil {
		misc.ErrorHandler(s, m.ChannelID, err)
		return allMessages
	}

	lastID = m.ID // just set lastid to this so that it wont delete the purge message and invoke message

	// first get the remainder of 100 because thats the max we can do at one time then do 100 each time.
	requestAmount := msgLimit % 100
	for count < msgLimit {
		msgList, err := s.ChannelMessages(m.ChannelID, requestAmount, lastID, "", "")
		msgIDList := make([]string, len(msgList))
		for i, msg := range msgList {
			msgIDList[i] = msg.ID
		}
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		}
		if len(msgList) == 0 {
			break
		}
		lastID = msgList[len(msgList)-1].ID
		err = s.ChannelMessagesBulkDelete(m.ChannelID, msgIDList)
		if err != nil {
			misc.ErrorHandler(s, m.ChannelID, err)
			return allMessages
		} else {
			allMessages = append(allMessages, msgList...)
		}
		count += len(msgList)
		if count == msgLimit {
			break
		}
		s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))

		// now we've done remainder, we can do 100 each time
		requestAmount = 100
	}
	// Update at the end with newest count before waiting and deleting
	s.ChannelMessageEdit(m.ChannelID, progressMsg.ID, fmt.Sprintf("Purging messages... [%v/%v]", count, msgLimit))

	time.Sleep(3 * time.Second)
	s.ChannelMessageDelete(m.ChannelID, m.ID)
	s.ChannelMessageDelete(m.ChannelID, progressMsg.ID)

	return allMessages
}

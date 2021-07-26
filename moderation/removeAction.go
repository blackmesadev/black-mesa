package moderation

import (
	"fmt"
	"log"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func RemoveActionCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, PERMISSION_REMOVEACTION) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	uuidList := misc.UuidRegex.FindAllString(m.Content, -1)

	if len(uuidList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `remove <action:uuid[]>`")
		return
	}
	unableRemove := make([]string, 0)
	msg := "<:mesaCheck:832350526729224243> Successfully removed "
	for _, uuid := range uuidList {
		deleteResult, err := config.RemoveAction(m.GuildID, uuid)
		if err != nil || deleteResult.DeletedCount < 1 {
			log.Println(err)
			unableRemove = append(unableRemove, uuid)
		} else {
			logging.LogRemoveAction(s, m.GuildID, m.Author.String(), uuid)
			msg += fmt.Sprintf("`%v` ", uuid)
		}
	}

	if len(unableRemove) != 0 {
		msg += "\n<:mesaCross:832350526414127195> Could not remove "
		for _, uuid := range unableRemove {
			msg += fmt.Sprintf("`%v` ", uuid)
		}

	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

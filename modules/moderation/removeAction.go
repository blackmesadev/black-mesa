package moderation

import (
	"fmt"
	"log"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func RemoveActionCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_REMOVEACTION) {
		db.NoPermissionHandler(s, m, conf, consts.PERMISSION_REMOVEACTION)
		return
	}

	start := time.Now()

	uuidList := util.UuidRegex.FindAllString(m.Content, -1)

	if len(uuidList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `remove <action:uuid[]>`")
		return
	}
	unableRemove := make([]string, 0)
	msg := "<:mesaCheck:832350526729224243> Successfully removed "
	for _, uuid := range uuidList {
		deleteResult, err := db.RemoveAction(m.GuildID, uuid)
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

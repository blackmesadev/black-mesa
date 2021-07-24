package moderation

import (
	"fmt"
	"log"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/discordgo"
)

func RemoveInfractionCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "moderation.remove") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	uuidList := misc.UuidRegex.FindAllString(m.Content, -1)

	if len(uuidList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `remove <action:uuid[]>`")
		return
	}
	unableRemove := make([]string, 0)
	msg := "<:mesaCheck:832350526729224243> Successfully removed "
	for _, uuid := range uuidList {
		ok, err := config.RemoveAction(m.GuildID, m.Author.ID)
		if err != nil || !ok {
			log.Println(err)
			unableRemove = append(unableRemove, uuid)
		} else {
			msg += fmt.Sprintf("`%v` ", uuid)
		}
	}

	if len(unableRemove) != 0 {
		msg += "\n<:mesaCross:832350526414127195> Could not remove "
		for _, uuid := range unableRemove {
			msg += fmt.Sprintf(`%v`, uuid)
		}

	}

	s.ChannelMessageSend(m.ChannelID, msg)

}

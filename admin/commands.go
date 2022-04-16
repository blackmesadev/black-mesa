package admin

import (
	"fmt"
	"strconv"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func LeaveCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !IsBotAdmin(m.Author.ID) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCross:832350526414127195> You do not have permission for that."))
		return
	}

	err := s.GuildLeave(m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCross:832350526414127195> Failed to leave server. `%v`", err))
		return
	}
}

func ForceLevelCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !IsBotAdmin(m.Author.ID) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("<:mesaCross:832350526414127195> You do not have permission for that."))
		return
	}

	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		idList = append(idList, m.Author.ID)
	}

	if len(args) < 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forcelevel <target:user[]> <level:int>`", consts.EMOJI_COMMAND))
		return
	}

	level := args[1]
	levelInt64, err := strconv.ParseInt(level, 10, 64)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forcelevel <target:user[]> <level:int>`", consts.EMOJI_COMMAND))
		return
	}

	err = db.SetLevel(s, m.GuildID, idList[0], levelInt64)

	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forcelevel <target:user[]> <level:int>`", consts.EMOJI_COMMAND))
		return
	}

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `%v` set to `%v` successfully.", consts.EMOJI_CHECK, idList[0], levelInt64))
}

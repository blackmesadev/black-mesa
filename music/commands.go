package music

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/discordgo"
)

func PlayCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must specify a URL.", consts.EMOJI_CROSS))
		return
	}

	ok := joinMemberChannel(s, m.ChannelID, m.GuildID, m.Author.ID)
	if !ok {
		return
	}

	playSong(s, m.ChannelID, m.GuildID, args[0])

}

func StopCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	err := stopSong(s, m.ChannelID, m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Stopped.", consts.EMOJI_CHECK))
	} else {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Failed to stop: %v", consts.EMOJI_CHECK, err))
	}

}

func DisconnectCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild.", consts.EMOJI_CROSS))
		return
	}

	stopSong(s, m.ChannelID, m.GuildID)

	s.ChannelVoiceLeave(m.GuildID)

}

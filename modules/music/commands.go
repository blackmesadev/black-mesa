package music

import (
	"fmt"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func PlayCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_PLAY)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}

	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	var arg string

	if len(args) == 0 {
		arg = ""
	} else {
		arg = strings.Join(args, " ")
	}

	// check if the bot is already in a voice channel
	botChannel, ok := isInUserVoiceChannel(s, m.GuildID, m.Author.ID)
	if !ok {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v I am already in a voice channel, join `<#%v>` to use me.", consts.EMOJI_CROSS, botChannel))
		return
	}

	ok = joinMemberChannel(s, m.ChannelID, m.GuildID, m.Author.ID)
	if !ok {
		return
	}

	playSong(s, m.ChannelID, m.GuildID, arg)

}

func StopCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_STOP)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	err := stopSong(s, m.ChannelID, m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, strings.Title(err.Error()))
	}

}

func SkipCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_SKIP)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	skipSong(s, m.ChannelID, m.GuildID)
}

func RemoveCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_REMOVE)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must specify a song index to remove", consts.EMOJI_CROSS))
		return
	}

	for _, arg := range args {
		argInt, err := strconv.Atoi(arg)
		if err != nil {
			s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `%v`", consts.EMOJI_CROSS, err.Error()))
			return
		}

		err = removeQueueByIndex(m.GuildID, argInt)
	}
}

func DisconnectCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_DC)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	s.ChannelVoiceLeave(m.GuildID)

	silentStop(s, m.GuildID)

	destroyPlayer(s, m.ChannelID, m.GuildID)

}

func NowPlayingCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_QUERY)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	nowPlaying(s, m.ChannelID, m.GuildID)

}

func SeekCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_SEEK)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `seek <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	seek(s, m.ChannelID, m.GuildID, args[0])
}

func ForwardCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_SEEK)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	currentDuration := getPosition(m.GuildID)
	if currentDuration == nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `An error occured while calculating duration.`", consts.EMOJI_CROSS))
		return
	}

	parsedDuration, err := time.ParseDuration(args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `forward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	newDuration := *currentDuration + parsedDuration

	rawSeek(s, m.ChannelID, m.GuildID, newDuration)

}

func BackwardCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_SEEK)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `backward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	currentDuration := getPosition(m.GuildID)
	if currentDuration == nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `An error occured while calculating duration.`", consts.EMOJI_CROSS))
		return
	}

	parsedDuration, err := time.ParseDuration(args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v `backward <time:duration>`", consts.EMOJI_COMMAND))
		return
	}

	newDuration := *currentDuration - parsedDuration

	rawSeek(s, m.ChannelID, m.GuildID, newDuration)

}

func VolumeCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_VOLUME)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	if len(args) == 0 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Volume: `%v`", consts.EMOJI_CHECK, getVolume(s, m.ChannelID, m.GuildID)))
		return
	}

	err := setVolume(s, m.ChannelID, m.GuildID, args[0])
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Failed to set Volume `%v`", consts.EMOJI_CROSS, err))
		return
	}
	// Use get volume here as a sort of check to the end user that it completed successfully.
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Set Volume to: `%v`", consts.EMOJI_CHECK, getVolume(s, m.ChannelID, m.GuildID)))

}

func QueueCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_QUERY)
	if !allowed {
		db.NoPermissionHandler(s, m, conf, perm)
		return
	}
	if m.GuildID == "" {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v You must execute this command in a guild", consts.EMOJI_CROSS))
		return
	}

	queue, err := getQueue(m.GuildID)
	if err != nil {
		if err == ErrEmptyQueue {
			s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v There is no queue!", consts.EMOJI_CROSS))
			return
		}
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("%v Unable to fetch queue `%v`", consts.EMOJI_CROSS, err))
		return
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
	}

	var guildName string
	guild, err := s.Guild(m.GuildID)
	if err != nil {
		guildName = m.GuildID
	} else {
		guildName = guild.Name
	}

	embedFields := make([]*discordgo.MessageEmbedField, 0)

	for i, track := range queue {
		_, duration := getTimeString(&track.Info)
		embedFields = append(embedFields, &discordgo.MessageEmbedField{
			Name:   util.ZeroWidth,
			Value:  fmt.Sprintf("`%v:` %v `(%v)`", i, track.Info.Title, duration),
			Inline: false,
		})
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Queue for %v", guildName),
		Fields: embedFields,
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
	}

	s.ChannelMessageSendEmbed(m.ChannelID, embed)

}

func AddQueueCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
}

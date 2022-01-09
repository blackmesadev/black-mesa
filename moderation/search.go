package moderation

import (
	"fmt"
	"log"
	"runtime"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func SearchCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	idList := util.SnowflakeRegex.FindAllString(m.Content, -1)

	uuidList := util.UuidRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 && len(uuidList) == 0 {
		idList = append(idList, m.Author.ID)
	}

	if idList[0] != m.Author.ID {
		for _, id := range idList {
			if id == m.Author.ID {
				if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_SEARCHSELF) {
					config.NoPermissionHandler(s, m, conf, consts.PERMISSION_SEARCHSELF)
					return
				}
			}
		}
	}

	if !config.CheckPermission(s, m.GuildID, m.Author.ID, consts.PERMISSION_SEARCH) {
		config.NoPermissionHandler(s, m, conf, consts.PERMISSION_SEARCH)
		return
	}

	if len(idList) > 1 || len(uuidList) > 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `search` takes 1 `[target:user]` or `[infraction:uuid]` parameter.")
		return
	}

	if len(idList) == 1 && len(uuidList) == 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `search` takes 1 `[target:user]` or `[infraction:uuid]` parameter.")
		return
	}

	var err error

	if len(idList) == 1 {
		_, err = SearchByUser(s, m, conf, idList,
			ShouldCensor(s, conf, m.GuildID, m.Author.ID))
	} else {
		_, err = SearchByUUID(s, m, conf, uuidList,
			ShouldCensor(s, conf, m.GuildID, m.Author.ID))
	}

	if err != nil {
		logging.LogError(s, m.GuildID, "", "SearchCmd", err)
	}

}

func SearchByUser(s *discordgo.Session, m *discordgo.Message, conf *structs.Config, idList []string, censored bool) (*discordgo.Message, error) {

	punishmentList, err := config.GetPunishments(m.GuildID, idList[0])
	if err != nil {
		log.Println(err)
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Could not search %v.")
	}

	embedFields := make([]*discordgo.MessageEmbedField, 0)

	for _, punishment := range punishmentList {
		var issuer string
		if punishment.Issuer != "AutoMod" {
			user, err := s.User(punishment.Issuer)
			if err != nil {
				issuer = punishment.Issuer
			} else {
				issuer = user.String()
			}
		} else {
			issuer = punishment.Issuer
		}
		var expiring string
		if punishment.Expires == 0 {
			expiring = "Never"
		} else {
			expiring = time.Unix(punishment.Expires, 0).Format(time.RFC3339)
		}

		if ShouldCensor(s, conf, m.GuildID, m.Author.ID) {
			punishment.Reason = util.FilteredCommand(punishment.Reason)
		}

		field := &discordgo.MessageEmbedField{
			Name: strings.Title(punishment.Type),
			Value: fmt.Sprintf("**Reason:** %v\n**Issued By:** %v\n**UUID:** %v\n**Expiring:** %v",
				punishment.Reason, issuer, punishment.UUID, expiring),
			Inline: true,
		}
		embedFields = append(embedFields, field)
	}
	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	user, err := s.User(idList[0])
	var userName string

	if err != nil {
		userName = idList[0]
	} else {
		userName = user.String()
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("%v's Infraction log.", userName),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}
	return s.ChannelMessageSendEmbed(m.ChannelID, embed)

}

func SearchByUUID(s *discordgo.Session, m *discordgo.Message, conf *structs.Config, uuidList []string, censored bool) (*discordgo.Message, error) {

	punishment, err := config.GetPunishmentByUUID(m.GuildID, uuidList[0])
	if err != nil {
		logging.LogError(s, m.GuildID, "", "SearchByUUID", err)
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 & LewisTehMinerz#1337 running on %v", info.VERSION, runtime.Version()),
	}

	var issuer string
	if punishment.Issuer != "AutoMod" {
		user, err := s.User(punishment.Issuer)
		if err != nil {
			issuer = punishment.Issuer
		} else {
			issuer = user.String()
		}
	} else {
		issuer = punishment.Issuer
	}
	embedFields := []*discordgo.MessageEmbedField{
		{
			Name: strings.Title(punishment.Type),
			Value: fmt.Sprintf("**Reason:** %v\n**Issued By:**%v\n**Expiring:** %v",
				util.FilteredCommand(punishment.Reason), issuer, time.Unix(punishment.Expires, 0)),
			Inline: true,
		},
	}

	embed := &discordgo.MessageEmbed{
		Title:  fmt.Sprintf("Infraction: %v", punishment.UUID),
		Type:   discordgo.EmbedTypeRich,
		Footer: footer,
		Color:  0, // Black int value
		Fields: embedFields,
	}

	return s.ChannelMessageSendEmbed(m.ChannelID, embed)
}

func ShouldCensor(s *discordgo.Session, conf *structs.Config, guildid string, userid string) bool {
	if conf == nil {
		var err error
		conf, err = config.GetConfig(guildid)
		if err != nil {
			log.Printf("Failed to get config for %v (%v)\n", guildid, err)
			return false
		}
	}

	if conf.Modules.Moderation.CensorStaffSearches && config.IsStaff(s, conf, guildid, userid) {
		return true
	}

	return conf.Modules.Moderation.CensorSearches
}

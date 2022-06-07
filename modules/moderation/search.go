package moderation

import (
	"fmt"
	"log"
	"runtime"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/consts"
	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/modules/automod/censor"
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

	selfSearch := true
	for _, id := range idList {
		if id == m.Author.ID {
			perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_SEARCHSELF)
			if !allowed {
				db.NoPermissionHandler(s, m, conf, perm)
				return
			}
		} else {
			selfSearch = false
		}
	}

	if !selfSearch {
		perm, allowed := db.CheckPermission(s, conf, m.GuildID, m.Author.ID, consts.PERMISSION_SEARCH)
		if !allowed {
			db.NoPermissionHandler(s, m, conf, perm)
			return
		}
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

	punishmentList, err := db.GetPunishments(m.GuildID, idList[0])
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
			expiring = fmt.Sprintf("<t:%v:f>", punishment.Expires)
		}

		if ShouldCensor(s, conf, m.GuildID, m.Author.ID) {
			punishment.Reason = util.FilteredCommand(punishment.Reason)
		}

		field := &discordgo.MessageEmbedField{
			Name: strings.Title(punishment.Type),
			Value: fmt.Sprintf("**Reason:** %v\n**Issued By:** %v\n**UUID:** %v\n**Expiring:** %v\n**Created:** <t:%v:f>",
				punishment.Reason, issuer, punishment.UUID, expiring, punishment.ID.Timestamp().Unix()),
			Inline: true,
		}
		embedFields = append(embedFields, field)
	}
	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
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

	punishment, err := db.GetPunishmentByUUID(m.GuildID, uuidList[0])
	if err != nil {
		logging.LogError(s, m.GuildID, "", "SearchByUUID", err)
	}

	footer := &discordgo.MessageEmbedFooter{
		Text: fmt.Sprintf("Black Mesa %v by Tyler#0911 running on %v", info.VERSION, runtime.Version()),
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

	var expiring string
	if punishment.Expires == 0 {
		expiring = "Never"
	} else {
		expiring = fmt.Sprintf("<t:%v:f>", time.Unix(punishment.Expires, 0))
	}

	if ShouldCensor(s, conf, m.GuildID, m.Author.ID) {
		userLevel := db.GetLevel(s, conf, m.GuildID, m.Author.ID)
		censorStruct := util.MakeCompleteCensorStruct(conf.Modules.Automod, m.ChannelID, userLevel)
		ok, str := censor.StringsCheck(punishment.Reason, censorStruct.BlockedStrings)
		if !ok {
			punishment.Reason = strings.Replace(punishment.Reason, str, util.FilteredTrigger(str), -1)
		}
		ok, str = censor.SubStringsCheck(punishment.Reason, censorStruct.BlockedSubstrings)
		if !ok {
			punishment.Reason = strings.Replace(punishment.Reason, str, util.FilteredTrigger(str), -1)
		}
	}

	embedFields := []*discordgo.MessageEmbedField{
		{
			Name: strings.Title(punishment.Type),
			Value: fmt.Sprintf("**Reason:** %v\n**Issued By:**%v\n**Expiring:** %v\n**Created:** <t:%v:f>",
				punishment.Reason, issuer, expiring, punishment.ID.Timestamp().Unix()),
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
		conf, err = db.GetConfig(guildid)
		if err != nil {
			log.Printf("Failed to get config for %v (%v)\n", guildid, err)
			return false
		}
	}
	if db.IsStaff(s, conf, guildid, userid) {
		return conf.Modules.Moderation.CensorStaffSearches
	}

	return conf.Modules.Moderation.CensorSearches
}

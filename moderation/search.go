package moderation

import (
	"fmt"
	"log"
	"runtime"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/info"
	"github.com/blackmesadev/black-mesa/misc"
	"github.com/blackmesadev/discordgo"
)

func SearchCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, PERMISSION_SEARCH) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	idList := misc.SnowflakeRegex.FindAllString(m.Content, -1)

	if len(idList) == 0 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `search <target:user[]>`")
		return
	}

	if len(idList) > 1 {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `search` takes 1 `<target:user[]>` parameter.")
		return
	}

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
		field := &discordgo.MessageEmbedField{
			Name:   punishment.Type,
			Value:  fmt.Sprintf("**Reason:** %v\n**Issued By:** %v\n**UUID:** %v", punishment.Reason, issuer, punishment.UUID),
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
	_, err = s.ChannelMessageSendEmbed(m.ChannelID, embed)
}

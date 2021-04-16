package discord

import "log"

func (bot *Bot) GetUserRolesID(guildid string, userid string) *[]string {

	m, err := bot.Session.GuildMember(guildid, userid)
	if err != nil {
		log.Println(err)
		return nil
	}

	roles := m.Roles

	return &roles
}

package config

import (
	"fmt"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

func MakeMuteCmd(s *discordgo.Session, conf *structs.Config, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !CheckPermission(s, m.GuildID, m.Author.ID, "admin") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	role, err := s.GuildRoleCreate(m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, err.Error())
	}

	// 8421504 = Grey Colour Int
	role, err = s.GuildRoleEdit(m.GuildID, role.ID, "Muted", 8421504, false, 0, false)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, err.Error())
		return
	}

	channelList, err := s.GuildChannels(m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, err.Error())
		return
	}

	textMutedOverwrite := &discordgo.PermissionOverwrite{
		ID:    role.ID,
		Type:  discordgo.PermissionOverwriteTypeRole,
		Deny:  2048, // Send Messages
		Allow: 0,    // Nothing to add
	}

	voiceMutedOverwrite := &discordgo.PermissionOverwrite{
		ID:    role.ID,
		Type:  discordgo.PermissionOverwriteTypeRole,
		Deny:  2097152, // Speak
		Allow: 0,       // Nothing to add

	}

	var (
		textCompleted  int
		voiceCompleted int
	)

	for _, channel := range channelList {
		var err error

		if channel.Type == discordgo.ChannelTypeGuildText {
			overwrites := channel.PermissionOverwrites
			overwrites = append(overwrites, textMutedOverwrite)
			edit := &discordgo.ChannelEdit{
				PermissionOverwrites: overwrites,
			}

			_, err = s.ChannelEditComplex(channel.ID, edit)
			if err != nil {
				s.ChannelMessageSend(m.ChannelID, err.Error())
				return
			}
			textCompleted++
		}

		if channel.Type == discordgo.ChannelTypeGuildVoice {
			overwrites := channel.PermissionOverwrites
			overwrites = append(overwrites, voiceMutedOverwrite)
			edit := &discordgo.ChannelEdit{
				PermissionOverwrites: overwrites,
			}

			_, err = s.ChannelEditComplex(channel.ID, edit)
			if err != nil {
				s.ChannelMessageSend(m.ChannelID, err.Error())
				return
			}
			voiceCompleted++
		}

	}

	updates, _ := db.SetConfigOne(m.GuildID, "modules.moderation.muteRole", role.ID)

	msg := fmt.Sprintf("<:mesaCheck:832350526729224243> Created role 'Muted' `(%v)`. Updated `%v` Text Channels and `%v` Voice Channels. Updated `%v` database entry.",
		role.ID, textCompleted, voiceCompleted, updates.ModifiedCount)

	s.ChannelMessageSend(m.ChannelID, msg)

}

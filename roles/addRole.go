package roles

import (
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/config"
	"github.com/blackmesadev/black-mesa/logging"
	"github.com/blackmesadev/black-mesa/util"
	"github.com/blackmesadev/discordgo"
)

func AddRoleCmd(s *discordgo.Session, m *discordgo.Message, ctx *discordgo.Context, args []string) {
	if !config.CheckPermission(s, m.GuildID, m.Author.ID, "roles.add") {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You do not have permission for that.")
		return
	}

	start := time.Now()

	var permRole bool

	var roleid string

	//idList, duration, reason := parseCommand(m.Content)
	idList := make([]string, 0)
	roleIdList := make([]string, 0)

	var argsRoleString string

	for _, possibleId := range args {
		if !util.UserIdRegex.MatchString(possibleId) {
			argsRoleString += possibleId + " "
			break
		} else {
			id := util.UserIdRegex.FindStringSubmatch(possibleId)[1]
			idList = append(idList, id)
		}

		if !util.RoleIdRegex.MatchString(possibleId) {
			argsRoleString += possibleId + " "
			break
		} else {
			roleid = util.RoleIdRegex.FindStringSubmatch(possibleId)[1]
			roleIdList = append(roleIdList, roleid)
		}
	}

	if len(idList) == 0 { // if there's no ids or the duration/reason start point is 0 for some reason
		s.ChannelMessageSend(m.ChannelID, "<:mesaCommand:832350527131746344> `addrole <target:user[]> [role:string] [time:duration]`")
		return
	}

	if !config.CheckTargets(s, m.GuildID, m.Author.ID, idList) {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> You can not target one or more of these users.")
		return
	}

	roles, err := s.GuildRoles(m.GuildID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, "<:mesaCross:832350526414127195> Unable to fetch roles.")
		return
	}

	for _, role := range roles {
		if strings.Contains(argsRoleString, role.Name) {
			roleIdList = append(roleIdList, role.ID)
		}
	}

	var duration int64

	if len(roleIdList) > 0 {
		if args[len(args)-1] == roleIdList[len(roleIdList)-1] {
			duration = 0
			permRole = true
		} else {
			duration = util.ParseTime(args[len(args)-1])
			permRole = false

		}
	} else if len(idList) > 0 {
		if args[len(args)-1] == idList[len(idList)-1] {
			duration = 0
			permRole = true
		} else {
			duration = util.ParseTime(args[len(args)-1])
			permRole = false

		}
	}

	msg := "<:mesaCheck:832350526729224243> Successfully added roles to "

	fullName := m.Author.Username + "#" + m.Author.Discriminator
	unableAddRole := make([]string, 0)
	for _, id := range idList {
		for _, roleid := range roleIdList {
			var member *discordgo.Member
			err := s.GuildMemberRoleAdd(m.GuildID, id, roleid) // change this to WithReason when implemented
			if err != nil {
				log.Println(err)
				unableAddRole = append(unableAddRole, id)
			} else {
				msg += fmt.Sprintf("<@%v> ", id)

				member, err = s.State.Member(m.GuildID, id)
				if err == discordgo.ErrStateNotFound {
					member, err = s.GuildMember(m.GuildID, id)
					if err != nil {
						log.Println(err)
						unableAddRole = append(unableAddRole, id)
					} else {
						s.State.MemberAdd(member)
					}
				}
			}

			var role *discordgo.Role
			for _, i := range roles {
				if i.ID == roleid {
					role = i
					break
				}
			}
			if permRole {
				AddRole(m.GuildID, m.Author.ID, id, roleid)
				logging.LogRoleAdd(s, m.GuildID, fullName, role.Name, member.User, m.ChannelID)
			} else {
				AddTimedRole(m.GuildID, m.Author.ID, id, roleid, duration)
				logging.LogTempRoleAdd(s, m.GuildID, fullName, role.Name, member.User, time.Duration(duration), m.ChannelID)
			}
		}
		if permRole {
			msg += "lasting `Forever` "
		} else {
			timeExpiry := time.Unix(duration, 0)
			timeUntil := time.Until(timeExpiry).Round(time.Second)
			msg += fmt.Sprintf("expiring `%v` (`%v`) ", timeExpiry, timeUntil.String())
		}
	}

	if len(unableAddRole) != 0 {
		msg += fmt.Sprintf("\n<:mesaCross:832350526414127195> Could not add roles to %v", unableAddRole)
	}

	s.ChannelMessageSend(m.ChannelID, msg)

	if util.IsDevInstance(s) {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("Operation completed in %v", time.Since(start)))
	}
}

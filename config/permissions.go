package config

import (
	"encoding/json"
	"log"
	"strings"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
)

// The concept here is that it will look for the most specific permission
// as in moderation.kick may be set to 25 where moderation is set to 50,
// in this case, kick will be 25. If moderation.kick is undefined, use the value at moderation.
// If moderation is not defined, if modules.guild.unsafePermissions is false then access will be
// denied, otherwise access will be granted.

func GetPermissions(s *discordgo.Session, guildid string, permission string) (int64, bool) {

	projection := make([]string, 1)

	permissionTree := strings.Split(permission, ".")

	projection[0] = "permissions." + permissionTree[0] // get the lowest node in the tree, makes it easier to step forward.

	data, err := db.GetConfigMultipleProjection(guildid, projection)
	if err != nil {
		return 0, false
	}

	bytesData, err := json.Marshal(data)
	if err != nil {
		return 0, false
	}

	var permissions map[string]int64

	err = json.Unmarshal(bytesData, &permissions)
	if err != nil {
		return 0, false
	}

	var permissionValue int64
	permissionValue = -1
	for _, v := range permissionTree {
		val, ok := permissions[v]
		if ok {
			permissionValue = val
		}
	}
	if permissionValue == -1 {
		return 0, false
	}
	return permissionValue, true
}

func GetLevel(s *discordgo.Session, guildid string, userid string) int64 {

	var levels map[string]int64

	data, err := db.GetConfigProjection(guildid, "levels")
	dataBytes, err := json.Marshal(data)
	if err != nil {
		log.Println(err)
		return -1
	}

	err = json.Unmarshal(dataBytes, &levels)

	if err != nil {
		log.Println(err)
		return -1
	}

	// first try userids only
	for k, v := range levels {
		if k == userid {
			return v
		}
	}

	// get roles instead then

	m, err := s.GuildMember(guildid, userid)
	if err != nil {
		log.Println(err)
		return -1
	}

	var highestLevel int64
	highestLevel = 0

	for _, role := range m.Roles {
		level, ok := levels[role]
		if ok {
			if level > highestLevel {
				highestLevel = level
			}
		}
	}

	return highestLevel
}

func CheckPermission(s *discordgo.Session, guildid string, userid string, permission string) bool {
	permissionValue, ok := GetPermissions(s, guildid, permission)
	if !ok {
		data, err := db.GetConfigProjection(guildid, "modules.guild.safePermissions")
		if err != nil {
			log.Println(err)
			return false
		}
		dataBytes, err := json.Marshal(data)
		if err != nil {
			log.Println(err)
			return false
		}

		guildStruct := &structs.Guild{}
		err = json.Unmarshal(dataBytes, &guildStruct)
		if err != nil {
			log.Println(err)
			return false
		}

		return guildStruct.UnsafePermissions
	}

	userLevel := GetLevel(s, guildid, userid)

	return userLevel >= permissionValue

}

package config

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"strings"
	"time"

	"github.com/blackmesadev/black-mesa/structs"
	"github.com/blackmesadev/discordgo"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/mongo"
)

// The concept here is that it will look for the most specific permission
// as in moderation.kick may be set to 25 where moderation is set to 50,
// in this case, kick will be 25. If moderation.kick is undefined, use the value at moderation.
// If moderation is not defined, if modules.guild.unsafePermissions is false then access will be
// denied, otherwise access will be granted.

func GetPermission(s *discordgo.Session, guildid string, permission string) (int64, error) {

	permissionTree := make([]string, 0)

	tempTree := strings.Split(permission, ".")

	for pk := range tempTree {
		node := tempTree[0]
		for i := 1; i <= pk; i++ {
			node += fmt.Sprintf(".%v", tempTree[i])
		}
		permissionTree = append(permissionTree, node)
	}

	data, err := db.GetConfigProjection(guildid, "permissions")
	if err != nil {
		log.Println(err)
		return 0, err
	}

	delete(data, "_id")

	conf := &structs.Config{}
	confBytes, err := json.Marshal(data["config"])
	if err != nil {
		log.Println(err)
		return 0, err
	}

	err = json.Unmarshal(confBytes, &conf)
	if err != nil {
		log.Println(err)
		return 0, err
	}

	var permissionValue int64
	permissionValue = -1
	// will iterate over the permissionTree in order so the least specific node comes first
	// and if anything more specific follows, will overwrite the permissionValue

	for _, v := range permissionTree {
		val, ok := conf.Permissions[v]
		if ok {
			permissionValue = val
		}
	}
	if permissionValue == -1 {
		err = errors.New("no data found")
		return 0, err
	}
	return permissionValue, nil
}

func GetLevel(s *discordgo.Session, guildid string, userid string) int64 {

	data, err := db.GetConfigProjection(guildid, "levels")
	if err != nil {
		log.Println(err)
		return 0
	}

	delete(data, "_id")

	conf := &structs.Config{}
	confBytes, err := json.Marshal(data["config"])

	if err != nil {
		log.Println(err)
		return 0
	}
	err = json.Unmarshal(confBytes, &conf)

	if err != nil {
		log.Println(err)
		return 0
	}

	// first try userids only
	for k, v := range conf.Levels {
		if k == userid {
			return v
		}
	}

	// get roles instead then

	m, err := s.GuildMember(guildid, userid)
	if err != nil {
		log.Println(err)
		return 0
	}

	var highestLevel int64
	highestLevel = 0

	for _, role := range m.Roles {
		level, ok := conf.Levels[role]
		if ok {
			if level > highestLevel {
				highestLevel = level
			}
		}
	}

	return highestLevel
}

func SetLevel(s *discordgo.Session, guildid string, userid string, level int64) error {
	col := db.GetMongoClient().Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": guildid}

	// If level is set to zero or below, just remove it
	if level > 0 {

		update := &bson.M{"$set": bson.M{"config.levels." + userid: level}}

		_, err := col.UpdateOne(ctx, filters, update)
		if err != nil {
			if err != mongo.ErrNoDocuments {
				log.Println(err)
			}
			return err
		}
	} else {
		filter := &bson.M{"guildID": guildid}
		update := &bson.M{"$pull": bson.M{"config.levels": userid}}

		_, err := col.UpdateOne(ctx, filter, update)
		if err != nil {
			if err != mongo.ErrNoDocuments {
				log.Println(err)
			}
		}
	}
	return nil
}

func SetPermission(s *discordgo.Session, guildid string, permission string, level int64) error {
	col := db.GetMongoClient().Database("black-mesa").Collection("guilds")
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()

	filters := &bson.M{"guildID": guildid}

	update := &bson.M{"$set": bson.M{"config.permissions." + permission: level}}

	_, err := col.UpdateOne(ctx, filters, update)
	if err != nil {
		if err != mongo.ErrNoDocuments {
			log.Println(err)
		}
		return err
	}
	return nil
}

func CheckPermission(s *discordgo.Session, guildid string, userid string, permission string) bool {
	var member *discordgo.Member
	member, err := s.State.Member(guildid, userid)
	if err != nil {
		if err == discordgo.ErrStateNotFound {
			member, err = s.GuildMember(guildid, userid)
			if err != nil {
				log.Println(err)
				return false
			}
			s.State.MemberAdd(member)
		}
	}
	if err != nil || member == nil {
		log.Println("failed to check permissions in", guildid, "for user", userid, "because", err)
		fmt.Println(member)
		return false // safety
	}

	for _, roleid := range member.Roles {
		role, err := s.State.Role(guildid, roleid)
		if err != nil {
			continue // skip
		}
		if role.Permissions&8 == 8 {
			return true // user is admin
		}
	}

	permissionValue, err := GetPermission(s, guildid, permission)
	if err != nil {
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

func CheckTargets(s *discordgo.Session, guildid string, actioner string, targets []string) bool {
	actionPermValue := GetLevel(s, guildid, actioner)

	for _, target := range targets {
		targetPermValue := GetLevel(s, guildid, target)
		if targetPermValue >= actionPermValue {
			return false
		}
	}
	return true
}

package util

import (
	"fmt"
	"log"
	"strconv"
	"strings"

	"github.com/blackmesadev/black-mesa/db"
	"github.com/blackmesadev/black-mesa/structs"
)

type ReasonVariable string

var (
	ReasonVariableModule               ReasonVariable = "{{module}}"
	ReasonVariableFunction             ReasonVariable = "{{function}}"
	ReasonVariableTrigger              ReasonVariable = "{{trigger}}"
	ReasonVariableTriggerCensored      ReasonVariable = "{{trigger_censored}}"
	ReasonVariableStrikeNo             ReasonVariable = "{{strike_no}}"
	ReasonVariableStrikesUntil         ReasonVariable = "{{strikes_until}}"
	ReasonVariableNextPunishment       ReasonVariable = "{{next_punishment}}"
	ReasonVariableNextPunishmentLength ReasonVariable = "{{next_punishment_length}}"
)

var allReasons = []ReasonVariable{
	ReasonVariableModule,
	ReasonVariableFunction,
	ReasonVariableTrigger,
	ReasonVariableTriggerCensored,
	ReasonVariableStrikeNo,
	ReasonVariableStrikesUntil,
	ReasonVariableNextPunishment,
}

func MakeReason(conf *structs.Config, guildId, userId, module, function, trigger string) (reason string) {
	if conf.Modules.Automod.ReasonMessage != "" {
		punishmentList, err := db.GetPunishments(guildId, userId)
		if err != nil {
			log.Println(err)
			return
		}
		for _, variable := range allReasons {
			reason = ReplaceReasonVariable(conf, punishmentList, reason, variable, module, function, trigger)
		}
		if reason == "" {
			reason = "No reason provided"
		}
	}

	return fmt.Sprintf("%v->%v (%v)", module, function, trigger)
}

func ReplaceReasonVariable(conf *structs.Config, strikes []*db.Action, reason string, variable ReasonVariable, module, function, trigger string) string {
	switch variable {
	case ReasonVariableModule:
		return strings.Replace(reason, string(variable), module, -1)
	case ReasonVariableFunction:
		return strings.Replace(reason, string(variable), function, -1)
	case ReasonVariableTrigger:
		return strings.Replace(reason, string(variable), trigger, -1)
	case ReasonVariableTriggerCensored:
		return strings.Replace(reason, string(variable), FilteredTrigger(trigger), -1)
	case ReasonVariableStrikeNo:
		strikeNo := len(strikes) + 1
		return strings.Replace(reason, string(variable), strconv.Itoa(strikeNo), -1)

	case ReasonVariableStrikesUntil:
		strikeNo := len(strikes) + 1
		esc := conf.Modules.Moderation.StrikeEscalation
		var closestHigher int
		for num := range esc {
			if int(num) >= strikeNo {
				closestHigher = int(num)
				break
			}
		}
		strikesUntil := closestHigher - strikeNo
		return strings.Replace(reason, string(variable), strconv.Itoa(strikesUntil), -1)

	case ReasonVariableNextPunishment:
		strikeNo := len(strikes) + 1
		esc := conf.Modules.Moderation.StrikeEscalation
		var nextPunishment structs.StrikeEscalation
		for num, punishment := range esc {
			if int(num) >= strikeNo {
				nextPunishment = punishment
				break
			}
		}
		return strings.Replace(reason, string(variable), nextPunishment.Type, -1)

	case ReasonVariableNextPunishmentLength:
		strikeNo := len(strikes) + 1
		esc := conf.Modules.Moderation.StrikeEscalation
		var nextPunishment structs.StrikeEscalation
		for num, punishment := range esc {
			if int(num) >= strikeNo {
				nextPunishment = punishment
				break
			}
		}
		return strings.Replace(reason, string(variable), nextPunishment.Duration, -1)
	}

	return reason
}

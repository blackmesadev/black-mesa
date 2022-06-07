package util

import (
	"reflect"

	"github.com/blackmesadev/black-mesa/structs"
)

func MakeCompleteCensorStruct(automod *structs.Automod, channelID string, userLevel int64) (combined *structs.Censor) {
	censorChannel := automod.CensorChannels[channelID]

	i := 0
	automodCensorLevels := make([]int64, len(automod.CensorLevels))
	for k := range automod.CensorLevels {
		automodCensorLevels[i] = k
		i++
	}

	censorStruct := automod.CensorLevels[GetClosestLevel(automodCensorLevels, userLevel)]

	if censorChannel == nil {
		return censorStruct
	}

	combined = censorChannel

	cv := reflect.ValueOf(combined).Elem()
	lv := reflect.ValueOf(censorStruct).Elem()

	for i := 0; i < cv.NumField(); i++ {
		cvf := cv.Field(i)
		// Do not include boolean in combination as we should be prioritising the channel settings anyway and IsZero() will return true for a false bool
		if cvf.IsZero() && cvf.Kind() != reflect.Bool {
			cvf.Set(lv.Field(i))
		}
	}
	return combined
}

func MakeCompleteSpamStruct(automod *structs.Automod, channelID string, userLevel int64) (combined *structs.Spam) {
	spamChannel := automod.SpamChannels[channelID]

	i := 0
	automodSpamLevels := make([]int64, len(automod.SpamLevels))
	for k := range automod.SpamLevels {
		automodSpamLevels[i] = k
		i++
	}

	spamLevel := automod.SpamLevels[GetClosestLevel(automodSpamLevels, userLevel)]

	if spamChannel == nil {
		return spamLevel
	}

	combined = spamChannel

	cv := reflect.ValueOf(combined).Elem()
	lv := reflect.ValueOf(spamLevel).Elem()

	for i := 0; i < cv.NumField(); i++ {
		cvf := cv.Field(i)
		// Do not include boolean in combination as we should be prioritising the channel settings and IsZero() will return true for a false bool
		if cvf.IsZero() && cvf.Kind() != reflect.Bool {
			cvf.Set(lv.Field(i))
		}
	}
	return combined
}

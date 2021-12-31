package util

import (
	"fmt"
	"strings"

	"github.com/blackmesadev/black-mesa/consts"
)

func FilteredCommand(input string) (output string) {
	blockedString := input

	if strings.Contains(input, consts.CENSOR_STRINGS) {
		blockedString = strings.TrimSpace(strings.TrimPrefix(input, consts.CENSOR_STRINGS))
		outputLength := len(blockedString)
		output = fmt.Sprintf("%v `(%c%v%c)`", consts.CENSOR_STRINGS, blockedString[1], strings.Repeat("*", outputLength-4), blockedString[outputLength-2])
	}

	if strings.Contains(input, consts.CENSOR_SUBSTRINGS) {
		blockedString = strings.TrimSpace(strings.TrimPrefix(input, consts.CENSOR_SUBSTRINGS))
		outputLength := len(blockedString)
		output = fmt.Sprintf("%v `(%c%v%c)`", consts.CENSOR_SUBSTRINGS, blockedString[1], strings.Repeat("*", outputLength-4), blockedString[outputLength-2])
	}

	return
}

package util

import (
	"fmt"
	"strings"

	"github.com/blackmesadev/black-mesa/consts"
)

func FilteredCommand(input string) (output string) {
	var blockedString string
	output = input // incase nothing is to be filtered

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

	if strings.Contains(input, consts.CENSOR_REGEX) {
		blockedString = strings.TrimSpace(strings.TrimPrefix(input, consts.CENSOR_REGEX))
		outputLength := len(blockedString)
		var outputString string
		repeatLength := outputLength - 4
		// total length of 16
		if repeatLength > 14 {
			outputString = strings.Repeat("*", 11) + "..."
		} else {
			outputString = strings.Repeat("*", repeatLength)
		}
		output = fmt.Sprintf("%v `(%c%v%c)`", consts.CENSOR_REGEX, blockedString[1], outputString, blockedString[outputLength-2])
	}

	return
}

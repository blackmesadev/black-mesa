package util

import (
	"fmt"
	"strings"
)

func FilteredCommand(input string) (output string) {
	var blockedString string

	if strings.HasPrefix(input, "Censor->BlockedString") {
		blockedString = strings.TrimPrefix(input, "Censor->BlockedString ")
	}

	if strings.HasPrefix(input, "Censor->BlockedSubString") {
		blockedString = strings.TrimPrefix(input, "Censor->BlockedSubString ")
	}

	outputLength := len(blockedString)
	output = fmt.Sprintf("*%c%v%c*", blockedString[1], strings.Repeat("*", outputLength-2), blockedString[outputLength-2])
	return
}

package censor

import (
	"regexp"
	"strings"
)

var nonStdSpaceRegex = regexp.MustCompile(`[\x{2000}-\x{200F}]+`)

func ReplaceNonStandardSpace(m string) string {
	return nonStdSpaceRegex.ReplaceAllString(m, " ")
}

func SubStringsCheck(m string, blacklist *[]string) (bool, string) {
	m = strings.ToLower(m)
	for _, substr := range *blacklist {
		if strings.Contains(m, substr) {
			return false, substr
		}
	}
	return true, ""
}

func StringsCheck(m string, blacklist *[]string) (bool, string) {
	m = strings.ToLower(m)
	words := strings.Fields(m)
	for _, str := range *blacklist {
		for _, word := range words {
			if word == str {
				return false, str
			}
		}
	}

	return true, ""
}
